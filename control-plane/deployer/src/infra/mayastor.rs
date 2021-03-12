use super::*;

#[async_trait]
impl ComponentAction for Mayastor {
    fn configure(
        &self,
        options: &StartOptions,
        cfg: Builder,
    ) -> Result<Builder, Error> {
        if options.build {
            let status = std::process::Command::new("cargo")
                .args(&["build", "-p", "mayastor", "--bin", "mayastor"])
                .status()?;
            build_error("mayastor", status.code())?;
        }

        if options.grpc_mayastor_ip.is_some() && options.mayastors > 1 {
            panic!("grpc_mayastor_ip and mayastors>1 not supported yet");
        }

        let mut cfg = cfg;
        for i in 0 .. options.mayastors {
            let mayastor_socket = format!("{}:10124", cfg.next_container_ip()?);

            let binary = Binary::from_nix("mayastor")
                .with_args(vec!["-N", &Self::name(i, options)]);

            let binary = if let Some(nats) = &options.nats_server {
                binary.with_args(vec!["-n", nats])
            } else {
                binary.with_nats("-n")
            };

            let binary = if let Some(grpc) = &options.grpc_mayastor_ip {
                binary.with_args(vec!["-g", grpc])
            } else {
                binary.with_args(vec!["-g", &mayastor_socket])
            };

            cfg = cfg.add_container_bin(&Self::name(i, options), binary)
        }
        Ok(cfg)
    }
    async fn start(
        &self,
        options: &StartOptions,
        cfg: &ComposeTest,
    ) -> Result<(), Error> {
        for i in 0 .. options.mayastors {
            cfg.start(&Self::name(i, options)).await?;
        }
        Ok(())
    }
    async fn wait_on(
        &self,
        options: &StartOptions,
        cfg: &ComposeTest,
    ) -> Result<(), Error> {
        for i in 0 .. options.mayastors {
            let mut hdl =
                cfg.grpc_handle(&Self::name(i, options)).await.unwrap();
            hdl.mayastor
                .list_nexus(rpc::mayastor::Null {})
                .await
                .unwrap();
        }
        Ok(())
    }
}

impl Mayastor {
    pub fn name(i: u32, options: &StartOptions) -> String {
        if options.mayastors == 1 {
            "mayastor".into()
        } else {
            format!("mayastor-{}", i + 1)
        }
    }
}
