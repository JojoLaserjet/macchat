//! ChakChat P2P Network Architecture
//!
//! Direct peer-to-peer communication via WebRTC DataChannels
//! Decentralized username discovery via DHT (Distributed Hash Table)
//! Zero central servers - completely decentralized!

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Peer information stored in DHT
#[derive(Clone, Debug)]
pub struct PeerInfo {
    /// Username (e.g., "alice@chakchat")
    pub username: String,

    /// Public key (for verification)
    pub public_key: Vec<u8>,

    /// Network endpoints (IP:Port, supports multiple)
    pub endpoints: Vec<SocketAddr>,

    /// Signature (proves ownership of private key)
    pub signature: Vec<u8>,

    /// Timestamp (for expiration)
    pub timestamp: i64,

    /// TTL in seconds (default 3600 = 1 hour)
    pub ttl: u32,
}

impl PeerInfo {
    /// Create new peer info
    pub fn new(
        username: String,
        public_key: Vec<u8>,
        endpoints: Vec<SocketAddr>,
        signature: Vec<u8>,
    ) -> Self {
        PeerInfo {
            username,
            public_key,
            endpoints,
            signature,
            timestamp: chrono::Local::now().timestamp(),
            ttl: 3600, // 1 hour default
        }
    }

    /// Check if peer info has expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Local::now().timestamp();
        let age = (now - self.timestamp) as u32;
        age > self.ttl
    }
}

/// DHT (Distributed Hash Table) Node
/// Every peer runs a DHT node for username discovery
pub struct DHTNode {
    /// Local peer ID
    pub peer_id: Vec<u8>,

    /// HashMap storing: username -> PeerInfo
    entries: Arc<RwLock<HashMap<String, PeerInfo>>>,

    /// Routing table for DHT lookups
    routing_table: Arc<RwLock<RoutingTable>>,
}

/// Simple Kademlia-style routing table
pub struct RoutingTable {
    /// K-buckets (distance-based routing)
    buckets: Vec<Vec<DHTNodeInfo>>,

    /// K value (replication factor)
    k: usize,
}

/// DHT node information for routing
#[derive(Clone, Debug)]
pub struct DHTNodeInfo {
    /// Node ID
    pub node_id: Vec<u8>,

    /// Network address
    pub addr: SocketAddr,

    /// Last seen timestamp
    pub last_seen: i64,
}

impl DHTNode {
    /// Create new DHT node
    pub fn new(peer_id: Vec<u8>) -> Self {
        DHTNode {
            peer_id,
            entries: Arc::new(RwLock::new(HashMap::new())),
            routing_table: Arc::new(RwLock::new(RoutingTable::new(20))),
        }
    }

    /// Publish peer info to DHT
    pub async fn publish(&self, peer_info: PeerInfo) -> Result<(), String> {
        // Verify signature before storing (TODO: implement signature verification)
        
        let mut entries = self.entries.write().await;
        entries.insert(peer_info.username.clone(), peer_info);
        Ok(())
    }

    /// Lookup peer by username
    pub async fn lookup(&self, username: &str) -> Result<Option<PeerInfo>, String> {
        let entries = self.entries.read().await;
        
        if let Some(peer_info) = entries.get(username) {
            // Check if expired
            if peer_info.is_expired() {
                drop(entries);
                // Remove expired entry
                self.entries.write().await.remove(username);
                return Ok(None);
            }
            return Ok(Some(peer_info.clone()));
        }

        Ok(None)
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) {
        let mut entries = self.entries.write().await;
        entries.retain(|_, peer_info| !peer_info.is_expired());
    }

    /// Get statistics about stored entries
    pub async fn stats(&self) -> DHTStats {
        let entries = self.entries.read().await;
        DHTStats {
            total_peers: entries.len(),
            active_peers: entries.values().filter(|p| !p.is_expired()).count(),
            expired_peers: entries.values().filter(|p| p.is_expired()).count(),
        }
    }
}

impl RoutingTable {
    /// Create new routing table with K value
    pub fn new(k: usize) -> Self {
        RoutingTable {
            buckets: vec![Vec::new(); 160], // 160 buckets for 160-bit distance
            k,
        }
    }

    /// Add node to routing table
    pub fn add_node(&mut self, node: DHTNodeInfo) {
        // Calculate bucket index based on XOR distance
        let bucket_index = Self::calculate_bucket_index(&node.node_id);
        
        if let Some(bucket) = self.buckets.get_mut(bucket_index) {
            // Remove if already exists
            bucket.retain(|n| n.node_id != node.node_id);
            
            // Add to front
            bucket.insert(0, node);
            
            // Keep only K nodes per bucket
            if bucket.len() > self.k {
                bucket.pop();
            }
        }
    }

    /// Get nearby nodes from routing table
    pub fn get_nearby_nodes(&self, target_id: &[u8], count: usize) -> Vec<DHTNodeInfo> {
        let mut nearby = Vec::new();
        
        for bucket in &self.buckets {
            for node in bucket {
                nearby.push(node.clone());
                if nearby.len() >= count {
                    return nearby;
                }
            }
        }
        
        nearby
    }

    /// Calculate bucket index based on XOR distance
    fn calculate_bucket_index(node_id: &[u8]) -> usize {
        if node_id.is_empty() {
            return 0;
        }
        
        // Simple calculation: use first byte modulo bucket count
        (node_id[0] as usize) % 160
    }
}

/// DHT statistics
#[derive(Debug)]
pub struct DHTStats {
    pub total_peers: usize,
    pub active_peers: usize,
    pub expired_peers: usize,
}

/// P2P Network Manager
pub struct P2PNetwork {
    /// DHT node
    dht: DHTNode,

    /// Active peer connections (TODO: WebRTC integration)
    peers: Arc<RwLock<HashMap<String, PeerConnection>>>,
}

/// Peer connection state
#[derive(Clone, Debug)]
pub struct PeerConnection {
    /// Peer username
    pub username: String,

    /// Connection status
    pub status: ConnectionStatus,

    /// Last activity timestamp
    pub last_activity: i64,

    /// Shared secret (for encryption)
    pub shared_secret: Option<Vec<u8>>,
}

/// Connection status
#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionStatus {
    /// Connecting
    Connecting,

    /// Connected
    Connected,

    /// Authenticated (handshake complete)
    Authenticated,

    /// Disconnected
    Disconnected,

    /// Error
    Error(String),
}

impl P2PNetwork {
    /// Create new P2P network
    pub fn new(peer_id: Vec<u8>) -> Self {
        P2PNetwork {
            dht: DHTNode::new(peer_id),
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Publish this peer to DHT
    pub async fn publish_self(&self, peer_info: PeerInfo) -> Result<(), String> {
        self.dht.publish(peer_info).await
    }

    /// Discover peer by username
    pub async fn discover_peer(&self, username: &str) -> Result<Option<PeerInfo>, String> {
        self.dht.lookup(username).await
    }

    /// Connect to peer by username
    pub async fn connect_to_peer(&self, username: &str) -> Result<(), String> {
        // 1. Lookup peer in DHT
        let peer_info = self
            .discover_peer(username)
            .await?
            .ok_or_else(|| format!("Peer not found: {}", username))?;

        // 2. Create connection record
        let connection = PeerConnection {
            username: username.to_string(),
            status: ConnectionStatus::Connecting,
            last_activity: chrono::Local::now().timestamp(),
            shared_secret: None,
        };

        // 3. Store connection
        let mut peers = self.peers.write().await;
        peers.insert(username.to_string(), connection);

        // TODO: 4. Establish WebRTC DataChannel
        // TODO: 5. Perform ECDH key exchange
        // TODO: 6. Mark as Authenticated

        Ok(())
    }

    /// Send message to peer (encrypted)
    pub async fn send_message(
        &self,
        username: &str,
        message: &[u8],
    ) -> Result<(), String> {
        let peers = self.peers.read().await;

        let connection = peers
            .get(username)
            .ok_or_else(|| format!("No connection to: {}", username))?;

        if connection.status != ConnectionStatus::Authenticated {
            return Err("Not authenticated with peer".to_string());
        }

        // TODO: Encrypt message with shared secret
        // TODO: Send via WebRTC DataChannel

        Ok(())
    }

    /// Get DHT statistics
    pub async fn get_stats(&self) -> DHTStats {
        self.dht.stats().await
    }

    /// Clean up expired entries
    pub async fn cleanup(&self) {
        self.dht.cleanup_expired().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_dht_publish_and_lookup() {
        let dht = DHTNode::new(b"peer1".to_vec());

        let peer_info = PeerInfo::new(
            "alice@chakchat".to_string(),
            b"pubkey123".to_vec(),
            vec![SocketAddr::from_str("192.168.1.1:8080").unwrap()],
            b"signature".to_vec(),
        );

        // Publish
        dht.publish(peer_info.clone()).await.unwrap();

        // Lookup
        let found = dht.lookup("alice@chakchat").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().username, "alice@chakchat");
    }

    #[tokio::test]
    async fn test_dht_lookup_nonexistent() {
        let dht = DHTNode::new(b"peer1".to_vec());

        let found = dht.lookup("nonexistent@chakchat").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_p2p_network_discovery() {
        let network = P2PNetwork::new(b"peer1".to_vec());

        let peer_info = PeerInfo::new(
            "bob@chakchat".to_string(),
            b"pubkey456".to_vec(),
            vec![SocketAddr::from_str("192.168.1.2:8081").unwrap()],
            b"signature2".to_vec(),
        );

        network.publish_self(peer_info).await.unwrap();

        let found = network.discover_peer("bob@chakchat").await.unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn test_dht_stats() {
        let dht = DHTNode::new(b"peer1".to_vec());

        let peer1 = PeerInfo::new(
            "alice".to_string(),
            b"key1".to_vec(),
            vec![],
            b"sig1".to_vec(),
        );

        let peer2 = PeerInfo::new(
            "bob".to_string(),
            b"key2".to_vec(),
            vec![],
            b"sig2".to_vec(),
        );

        dht.publish(peer1).await.unwrap();
        dht.publish(peer2).await.unwrap();

        let stats = dht.stats().await;
        assert_eq!(stats.total_peers, 2);
        assert_eq!(stats.active_peers, 2);
    }

    #[test]
    fn test_routing_table_add_node() {
        let mut rt = RoutingTable::new(20);

        let node = DHTNodeInfo {
            node_id: b"node1".to_vec(),
            addr: SocketAddr::from_str("192.168.1.1:8080").unwrap(),
            last_seen: chrono::Local::now().timestamp(),
        };

        rt.add_node(node);

        let nearby = rt.get_nearby_nodes(b"target", 5);
        assert_eq!(nearby.len(), 1);
    }
}
