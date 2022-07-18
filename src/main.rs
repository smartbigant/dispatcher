use std::time::Duration;
use bastion::prelude::*;
use bastion::distributed::*;
use lazy_static::*;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use uuid::Uuid;
use futures_timer::Delay;
use artillery_core::service_discovery::mdns::prelude::*;
use artillery_core::epidemic::cluster_config::ClusterConfig;

lazy_static! {
    static ref CLUSTER_CONFIG: ArtilleryAPClusterConfig = {
        // let port = get_port();
        ArtilleryAPClusterConfig {
            app_name: String::from("artillery-ap-world3"),
            node_id: Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"game-dispatcher-world3"),
            sd_config: {
                MDNSServiceDiscoveryConfig::default()
                // let mut config = MDNSServiceDiscoveryConfig::default();
                // config.local_service_addr.set_port(port);
                // config
            },
            cluster_config: {
                ClusterConfig::default()

                // let listen_addr = format!("127.0.0.1:{}", CONST_INFECTION_PORT);
                // ClusterConfig {
                //     listen_addr: (&listen_addr as &str)
                //         .to_socket_addrs()
                //         .unwrap()
                //         .next()
                //         .unwrap(),
                //     ..Default::default()
                // }
            },
        }
    };
}


fn main() {
    Bastion::init();

    println!("{:?}", CLUSTER_CONFIG.cluster_config);

    let instance = Bastion::distributed(
        &*CLUSTER_CONFIG,
        |dist_ctx: Arc<DistributedContext>| async move {
            let out_dist_ctx = dist_ctx.clone();
            let _outbound = blocking!(loop {
                out_dist_ctx.members().iter().for_each(|m| {
                    let message = format!("PING FROM {}", out_dist_ctx.current());
                    let _ = out_dist_ctx.tell(&m.host_key(), message);
                });

                let _member_msg_wait = Delay::new(Duration::from_secs(1)).await;
            });

            println!("Started listening...");
            loop {
                let msg = dist_ctx.recv().await?.extract();
                let member_msg: String = msg.downcast().unwrap();
                println!("Message received: {:?}", member_msg);
            }
        },
    ).expect("Couldn't start cluster node.");

    Bastion::start();
    Bastion::block_until_stopped();
}
