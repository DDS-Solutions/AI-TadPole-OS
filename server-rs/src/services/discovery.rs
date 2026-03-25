use crate::agent::types::SwarmNode;
use crate::state::AppState;
use chrono::Utc;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct SwarmDiscoveryManager {
    app_state: Arc<AppState>,
    daemon: ServiceDaemon,
}

impl SwarmDiscoveryManager {
    pub fn new(app_state: Arc<AppState>) -> anyhow::Result<Self> {
        let daemon = ServiceDaemon::new()?;
        Ok(Self { app_state, daemon })
    }

    pub fn start(&self) -> anyhow::Result<()> {
        let name = std::env::var("CLUSTER_ID").unwrap_or_else(|_| "tadpole-node".to_string());
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "8000".to_string())
            .parse::<u16>()
            .unwrap_or(8000);

        // 1. Register self
        let service_type = "_tadpole._tcp.local.";
        let node_id = Uuid::new_v4().to_string();
        let instance_name = format!("{}-{}", name.replace(" ", "-"), &node_id[..8]);
        let host_name = format!("{}.local.", instance_name);

        let mut properties = HashMap::new();
        properties.insert("id".to_string(), node_id.clone());
        properties.insert("name".to_string(), name.clone());
        properties.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());

        // We use 0.0.0.0 to signal the daemon to find the local IP if possible,
        // but mdns-sd might require a concrete one. For now, we'll try to get it.
        let my_ip = "127.0.0.1"; // Fallback

        let my_service = ServiceInfo::new(
            service_type,
            &instance_name,
            &host_name,
            my_ip,
            port,
            Some(properties),
        )?;

        self.daemon.register(my_service)?;
        tracing::info!(
            "📡 [Discovery] mDNS Service Registered: {} on port {}",
            instance_name,
            port
        );

        // 2. Browse for others
        let receiver = self.daemon.browse(service_type)?;
        let state = self.app_state.clone();

        tokio::spawn(async move {
            tracing::info!(
                "📡 [Discovery] Swarm Browser Active: Searching for _tadpole._tcp.local..."
            );
            while let Ok(event) = receiver.recv_async().await {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        let id = info
                            .get_property_val("id")
                            .flatten()
                            .map(|v| String::from_utf8_lossy(v).to_string())
                            .unwrap_or_else(|| info.get_fullname().to_string());
                        let name = info
                            .get_property_val("name")
                            .flatten()
                            .map(|v| String::from_utf8_lossy(v).to_string())
                            .unwrap_or_else(|| info.get_fullname().to_string());

                        let address = info
                            .get_addresses()
                            .iter()
                            .next()
                            .map(|ip| format!("{}:{}", ip, info.get_port()))
                            .unwrap_or_else(|| format!("127.0.0.1:{}", info.get_port()));

                        let mut metadata = HashMap::new();
                        for prop in info.get_properties().iter() {
                            metadata.insert(prop.key().to_string(), prop.val_str().to_string());
                        }

                        let node = SwarmNode {
                            id: id.clone(),
                            name,
                            address,
                            status: "online".to_string(),
                            last_seen: Utc::now(),
                            metadata,
                        };

                        state.registry.nodes.insert(id.clone(), node);
                        tracing::info!(
                            "🔗 [Discovery] Found swarm node: {} at {}",
                            id,
                            info.get_fullname()
                        );
                    }
                    ServiceEvent::ServiceRemoved(_type, name) => {
                        tracing::info!("🔌 [Discovery] Node removed: {}", name);
                        // We could mark as offline here but DashMap doesn't have an easy "find by name prefix"
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }
}
