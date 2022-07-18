use std::time::Duration;
use bastion::prelude::*;
use bastion::distributed::*;
use lazy_static::*;
use std::sync::Arc;
use uuid::Uuid;
use futures_timer::Delay;
use std::net::ToSocketAddrs;
use artillery_core::service_discovery::mdns::prelude::*;
use artillery_core::epidemic::cluster_config::ClusterConfig;
use log4rs::append::console::ConsoleAppender;
use log::{info, LevelFilter};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};

lazy_static! {
    static ref CLUSTER_CONFIG: ArtilleryAPClusterConfig = {
         let port = get_port();
        ArtilleryAPClusterConfig {
            app_name: String::from("artillery-ap-world3"),
            node_id: Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"game-dispatcher-world3"),
            sd_config: {
                //MDNSServiceDiscoveryConfig::default()
                let mut config = MDNSServiceDiscoveryConfig::default();
                config.local_service_addr.set_port(port);
                config
            },
            cluster_config: {
                //ClusterConfig::default()

                let listen_addr = format!("127.0.0.1:{}", port);
                ClusterConfig {
                    listen_addr: (&listen_addr as &str)
                        .to_socket_addrs()
                        .unwrap()
                        .next()
                        .unwrap(),
                    ..Default::default()
                }
            },
        }
    };
}


fn main() {
    let console = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build();
    let config = Config::builder()
        .appender(Appender::builder().build("console", Box::new(console)))
        .build(Root::builder()
            .appender("console")
            .build(LevelFilter::Debug)).unwrap();
    log4rs::init_config(config).unwrap();

    Bastion::init();

    info!("{:?}", CLUSTER_CONFIG.cluster_config);
    info!("{}", CLUSTER_CONFIG.node_id);
    info!("{:?}", CLUSTER_CONFIG.sd_config);

    let instance = Bastion::distributed(
        &*CLUSTER_CONFIG,
        |dist_ctx: Arc<DistributedContext>| async move {
            let out_dist_ctx = dist_ctx.clone();
            let _outbound = blocking!(loop {
                out_dist_ctx.members().iter().for_each(|m| {
                    let message = format!("PING FROM {}", out_dist_ctx.current());
                    let _ = out_dist_ctx.tell(&m.host_key(), message);
                });

                let _member_msg_wait = Delay::new(Duration::from_secs(10)).await;
            });

            info!("Started listening...");
            loop {
                let msg = dist_ctx.recv().await?.extract();
                let member_msg: String = msg.downcast().unwrap();
                info!("Message received: {:?}", member_msg);
            }
        },
    ).expect("Couldn't start cluster node.");

    Bastion::start();
    Bastion::block_until_stopped();
}

fn get_port() -> u16 {
    use rand::{thread_rng, Rng};

    let mut rng = thread_rng();
    let port: u16 = rng.gen();
    if port > 1025 && port < 65535 {
        port
    } else {
        get_port()
    }
}
