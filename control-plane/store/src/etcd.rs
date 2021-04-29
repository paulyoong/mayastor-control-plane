use crate::store::{
    Connect,
    Delete,
    DeserialiseValue,
    Get,
    KeyString,
    ObjectKey,
    Put,
    SerialiseValue,
    StorableObject,
    Store,
    StoreError,
    StoreError::MissingEntry,
    StoreKey,
    ValueString,
    Watch,
    WatchEvent,
};
use async_trait::async_trait;
use etcd_client::{Client, EventType, KeyValue, WatchStream, Watcher};
use serde_json::Value;
use snafu::ResultExt;
use tokio::sync::mpsc::{channel, Receiver, Sender};

/// etcd client
#[derive(Clone)]
pub struct Etcd(Client);

impl std::fmt::Debug for Etcd {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Etcd {
    /// Create a new instance of the etcd client
    pub async fn new(endpoint: &str) -> Result<Etcd, StoreError> {
        Ok(Self(
            Client::connect([endpoint], None)
                .await
                .context(Connect {})?,
        ))
    }
}

#[async_trait]
impl Store for Etcd {
    async fn put_obj<O: StorableObject>(&mut self, object: &O) -> Result<(), StoreError> {
        let key = object.key().key();
        let vec_value = serde_json::to_vec(object).context(SerialiseValue)?;
        self.0.put(key, vec_value, None).await.context(Put {
            key: object.key().key(),
            value: serde_json::to_string(object).context(SerialiseValue)?,
        })?;
        Ok(())
    }

    async fn get_obj<O: StorableObject>(&mut self, key: &O::Key) -> Result<O, StoreError> {
        let resp = self.0.get(key.key(), None).await.context(Get {
            key: key.key(),
        })?;
        match resp.kvs().first() {
            Some(kv) => Ok(
                serde_json::from_slice(kv.value()).context(DeserialiseValue {
                    value: kv.value_str().context(ValueString {})?,
                })?,
            ),
            None => Err(MissingEntry {
                key: key.key(),
            }),
        }
    }

    async fn get_opaque_obj<K: StoreKey>(&mut self, key: &K) -> Result<Value, StoreError> {
        let resp = self.0.get(key.to_string(), None).await.context(Get {
            key: key.to_string(),
        })?;
        match resp.kvs().first() {
            Some(kv) => Ok(
                serde_json::from_slice(kv.value()).context(DeserialiseValue {
                    value: kv.value_str().context(ValueString {})?,
                })?,
            ),
            None => Err(MissingEntry {
                key: key.to_string(),
            }),
        }
    }

    /// 'Delete' the entry with the given key from etcd.
    async fn delete_obj<K: ObjectKey>(&mut self, key: &K) -> Result<(), StoreError> {
        self.0.delete(key.key(), None).await.context(Delete {
            key: key.key(),
        })?;
        Ok(())
    }

    async fn watch_obj<K: ObjectKey>(
        &mut self,
        key: &K,
    ) -> Result<Receiver<Result<WatchEvent, StoreError>>, StoreError> {
        let (sender, receiver) = channel(100);
        let (watcher, stream) = self.0.watch(key.key(), None).await.context(Watch {
            key: key.key(),
        })?;
        watch(watcher, stream, sender);
        Ok(receiver)
    }

    async fn online(&mut self) -> bool {
        self.0.status().await.is_ok()
    }
}

/// Watch for events in the key-value store.
/// When an event occurs, a WatchEvent is sent over the channel.
/// When a 'delete' event is received, the watcher stops watching.
fn watch(
    _watcher: Watcher,
    mut stream: WatchStream,
    mut sender: Sender<Result<WatchEvent, StoreError>>,
) {
    // For now we spawn a thread for each value that is watched.
    // If we find that we are watching lots of events, this can be optimised.
    // TODO: Optimise the spawning of threads if required.
    tokio::spawn(async move {
        loop {
            let response = match stream.message().await {
                Ok(msg) => {
                    match msg {
                        Some(resp) => resp,
                        // stream cancelled
                        None => {
                            return;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to get message with error {}", e);
                    return;
                }
            };

            for event in response.events() {
                match event.event_type() {
                    EventType::Put => {
                        if let Some(kv) = event.kv() {
                            let result = match deserialise_kv(&kv) {
                                Ok((key, value)) => Ok(WatchEvent::Put(key, value)),
                                Err(e) => Err(e),
                            };
                            if sender.send(result).await.is_err() {
                                // Send only fails if the receiver is closed, so
                                // just stop watching.
                                return;
                            }
                        }
                    }
                    EventType::Delete => {
                        // Send only fails if the receiver is closed. We are
                        // returning here anyway, so the error doesn't need to
                        // be handled.
                        let _ = sender.send(Ok(WatchEvent::Delete)).await;
                        return;
                    }
                }
            }
        }
    });
}

/// Deserialise a key-value pair into serde_json::Value representations.
fn deserialise_kv(kv: &KeyValue) -> Result<(String, Value), StoreError> {
    let key_str = kv.key_str().context(KeyString {})?.to_string();
    let value_str = kv.value_str().context(ValueString {})?;
    let value = serde_json::from_str(value_str).context(DeserialiseValue {
        value: value_str.to_string(),
    })?;
    Ok((key_str, value))
}
