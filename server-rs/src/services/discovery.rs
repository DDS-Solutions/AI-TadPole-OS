//! @docs ARCHITECTURE:Networking:Discovery
//!
//! ### AI Context Alignment
//! - **Subsystem**: Frontend Service Layer / discovery
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Discovery]`
//! - **Witness Tests**: none declared

use crate::agent::types::SwarmNode;
use crate::state::AppState;
use chrono::Utc;
use dashmap::DashMap;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Helper to prune nodes from the swarm registry by their mDNS full instance name.
/// Shared between production mDNS removal events and integration tests.
pub fn prune_nodes_by_mdns_name(
    nodes: &DashMap<String, SwarmNode>,
    mdns_name: &str,
) -> Vec<String> {
    let to_remove: Vec<String> = nodes
        .iter()
        .filter(|entry| {
            entry
                .value()
                .metadata
                .get("mdns_name")
                .map(|n| n == mdns_name)
                .unwrap_or(false)
        })
        .map(|entry| entry.key().clone())
        .collect();

    for id in &to_remove {
        nodes.remove(id);
    }
    to_remove
}

pub struct SwarmDiscoveryManager {
    app_state: Arc<AppState>,
    daemon: ServiceDaemon,
}

impl SwarmDiscoveryManager {
    pub fn new(app_state: Arc<AppState>) -> anyhow::Result<Self> {
        let daemon = ServiceDaemon::new()?;
        Ok(Self { app_state, daemon })
    }

    pub fn start(&self, mut shutdown_rx: tokio::sync::watch::Receiver<bool>) -> anyhow::Result<()> {
        let name = std::env::var("CLUSTER_ID").unwrap_or_else(|_| "tadpole-node".to_string());
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "8000".to_string())
            .parse::<u16>()
            .unwrap_or(8000);

        // 1. Resolve local IPv4 non-loopback IP for LAN discovery
        let my_ip = if let Ok(ips) = local_ip_address::list_afinet_netifas() {
            ips.iter()
                .find(|(_name, ip)| ip.is_ipv4() && !ip.is_loopback())
                .map(|(_name, ip)| ip.to_string())
                .unwrap_or_else(|| "127.0.0.1".to_string())
        } else {
            "127.0.0.1".to_string()
        };

        let service_type = "_tadpole._tcp.local.";
        let node_id = Uuid::new_v4().to_string();
        let instance_name = format!("{}-{}", name.replace(' ', "-"), &node_id[..8]);
        let host_name = format!("{}.local.", instance_name);

        let mut properties = HashMap::new();
        properties.insert("id".to_string(), node_id.clone());
        properties.insert("name".to_string(), name.clone());
        properties.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());

        let my_service = ServiceInfo::new(
            service_type,
            &instance_name,
            &host_name,
            &my_ip,
            port,
            Some(properties),
        )?;
        let my_fullname = my_service.get_fullname().to_string();

        if let Err(e) = self.daemon.register(my_service) {
            tracing::error!("📡 [Discovery] Failed to register mDNS service: {}. Swarm discovery will be limited.", e);
        } else {
            tracing::info!(
                "📡 [Discovery] mDNS Service Registered: {} on {} (port {})",
                instance_name,
                my_ip,
                port
            );
        }

        // 2. Browse for peers
        let receiver = self.daemon.browse(service_type)?;
        let state = self.app_state.clone();
        let daemon = self.daemon.clone();
        let self_node_id = node_id.clone();
        let self_fullname = my_fullname.clone();

        tokio::spawn(async move {
            tracing::info!(
                "📡 [Discovery] Swarm Browser Active: Searching for _tadpole._tcp.local..."
            );
            loop {
                tokio::select! {
                    event_res = receiver.recv_async() => {
                        match event_res {
                            Ok(event) => {
                                match event {
                                    ServiceEvent::ServiceResolved(info) => {
                                        let id = info
                                            .get_property_val("id")
                                            .flatten()
                                            .map(|v| String::from_utf8_lossy(v).to_string())
                                            .unwrap_or_else(|| info.get_fullname().to_string());

                                        // Self-discovery filter: Do not insert self into peer registry
                                        if id == self_node_id || info.get_fullname() == self_fullname {
                                            continue;
                                        }

                                        let name = info
                                            .get_property_val("name")
                                            .flatten()
                                            .map(|v| String::from_utf8_lossy(v).to_string())
                                            .unwrap_or_else(|| info.get_fullname().to_string());

                                        // Prefer IPv4 non-loopback addresses
                                        let address = info
                                            .get_addresses()
                                            .iter()
                                            .find(|ip| ip.is_ipv4() && !ip.is_loopback())
                                            .or_else(|| info.get_addresses().iter().next())
                                            .map(|ip| format!("{}:{}", ip, info.get_port()))
                                            .unwrap_or_else(|| format!("127.0.0.1:{}", info.get_port()));

                                        let mut metadata: HashMap<String, String> = HashMap::new();
                                        for prop in info.get_properties().iter() {
                                            metadata.insert(prop.key().to_string(), prop.val_str().to_string());
                                        }
                                        // Store the mDNS fullname for cleanup on removal
                                        metadata.insert("mdns_name".to_string(), info.get_fullname().to_string());

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
                                            "📡 [Discovery] Found peer node: {} at {}",
                                            id,
                                            info.get_fullname()
                                        );
                                    }
                                    ServiceEvent::ServiceRemoved(_type, name) => {
                                        tracing::info!("🔌 [Discovery] Node removed: {}", name);
                                        let pruned = prune_nodes_by_mdns_name(&state.registry.nodes, &name);
                                        for id in pruned {
                                            tracing::info!("🗑️ [Discovery] Pruned node from registry: {}", id);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    res = shutdown_rx.changed() => {
                        match res {
                            Ok(_) => {
                                if *shutdown_rx.borrow() {
                                    tracing::info!("📡 [Discovery] Swarm Browser shutting down gracefully.");
                                    break;
                                }
                            }
                            Err(_) => {
                                tracing::info!("📡 [Discovery] Shutdown channel closed, terminating.");
                                break;
                            }
                        }
                    }
                }
            }

            // Cleanup mDNS registration on task shutdown
            if let Err(e) = daemon.unregister(&self_fullname) {
                tracing::warn!(
                    "📡 [Discovery] Failed to unregister mDNS service on shutdown: {}",
                    e
                );
            }
            if let Err(e) = daemon.stop_browse(service_type) {
                tracing::warn!(
                    "📡 [Discovery] Failed to stop mDNS browse on shutdown: {}",
                    e
                );
            }
        });

        Ok(())
    }
}
