use super::*;

#[async_trait]
impl ComponentAction for Nats {
    fn configure(
        &self,
        options: &StartOptions,
        cfg: Builder,
    ) -> Result<Builder, Error> {
        let cfg = if options.nats_server.is_none() {
            cfg.add_container_spec(
                ContainerSpec::from_binary(
                    "nats",
                    Binary::from_nix("nats-server").with_arg("-DV"),
                )
                .with_portmap("4222", "4222"),
            )
        } else {
            cfg
        };
        Ok(cfg)
    }
    async fn start(
        &self,
        options: &StartOptions,
        cfg: &ComposeTest,
    ) -> Result<(), Error> {
        if let Some(nats) = &options.nats_server {
            cfg.connect_to_bus(nats).await;
        } else {
            cfg.start("nats").await?;
            cfg.connect_to_bus("nats").await;
        }
        Ok(())
    }
}
