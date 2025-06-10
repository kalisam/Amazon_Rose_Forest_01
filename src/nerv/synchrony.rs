use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

type VectorClock = HashMap<String, u64>;

#[derive(Debug, Clone)]
struct SynchronyState {
    node_id: String,
    clock: VectorClock,
    last_sync: HashMap<String, chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug)]
pub struct SynchronyManager {
    state: RwLock<SynchronyState>,
    peers: RwLock<HashSet<String>>,
}

impl SynchronyManager {
    pub fn new(node_id: &str) -> Self {
        let mut clock = HashMap::new();
        clock.insert(node_id.to_string(), 0);

        let state = SynchronyState {
            node_id: node_id.to_string(),
            clock,
            last_sync: HashMap::new(),
        };

        Self {
            state: RwLock::new(state),
            peers: RwLock::new(HashSet::new()),
        }
    }

    pub async fn add_peer(&self, peer_id: &str) {
        self.peers.write().await.insert(peer_id.to_string());
        info!("Added peer {} to synchrony manager", peer_id);
    }

    pub async fn remove_peer(&self, peer_id: &str) {
        self.peers.write().await.remove(peer_id);
        info!("Removed peer {} from synchrony manager", peer_id);
    }

    pub async fn increment_local_clock(&self) -> u64 {
        let mut state = self.state.write().await;
        let node_id = state.node_id.clone();
        let counter = state.clock.entry(node_id).or_insert(0);


        *counter += 1;
        *counter
    }

    pub async fn merge_remote_clock(&self, remote_node: &str, remote_clock: VectorClock) {
        let mut state = self.state.write().await;

        // Update last sync time
        state
            .last_sync
            .insert(remote_node.to_string(), chrono::Utc::now());

        // Merge the clocks
        for (node, &timestamp) in &remote_clock {
            let local_timestamp = state.clock.entry(node.clone()).or_insert(0);
            *local_timestamp = std::cmp::max(*local_timestamp, timestamp);
        }

        info!("Merged clock from node {}", remote_node);
    }

    pub async fn get_current_clock(&self) -> VectorClock {
        self.state.read().await.clock.clone()
    }

    pub async fn is_causally_ready(&self, event_clock: &VectorClock) -> bool {
        let state = self.state.read().await;

        for (node, &timestamp) in event_clock {
            if node == &state.node_id {
                continue; // Skip our own node
            }

            let local_timestamp = state.clock.get(node).copied().unwrap_or(0);
            if local_timestamp < timestamp {
                // We haven't seen all the events this event depends on
                return false;
            }
        }

        true
    }

    pub async fn get_sync_status(&self) -> HashMap<String, (chrono::DateTime<chrono::Utc>, bool)> {
        let state = self.state.read().await;
        let peers = self.peers.read().await;

        let mut result = HashMap::new();
        let now = chrono::Utc::now();

        // Threshold for considering a node out of sync (5 minutes)
        let threshold = chrono::Duration::minutes(5);

        for peer in peers.iter() {
            let last_sync = state.last_sync.get(peer).copied().unwrap_or_else(|| {
                // If we've never synced, use a very old time
                chrono::Utc::now() - chrono::Duration::days(365)
            });

            let is_in_sync = now - last_sync < threshold;
            result.insert(peer.clone(), (last_sync, is_in_sync));
        }

        result
    }
}
