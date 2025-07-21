pub mod consciousness;
pub mod core;
pub mod darwin;
pub mod governance;
pub mod intelligence;
pub mod nerv;
pub mod network;
pub mod server;
pub mod sharding;
pub mod utils;
pub mod llm;
pub mod code_analysis;
pub mod hypothesis;
pub mod evaluation;
pub mod ad4m;

// Export common types for easier access
pub use crate::consciousness::ad4m_bridge::Ad4mBridge;
pub use crate::consciousness::introspection::Introspection;
pub use crate::consciousness::swarm::Swarm;
pub use crate::core::centroid::Centroid;
pub use crate::core::centroid_crdt::CentroidCRDT;
pub use crate::core::vector::Vector;
pub use crate::darwin::self_improvement::SelfImprovementEngine;
pub use crate::governance::dao::Dao;
pub use crate::governance::zkp::ZKP;
pub use crate::intelligence::federated_learning::FederatedLearning;
pub use crate::intelligence::orchestrator::Orchestrator;
pub use crate::nerv::runtime::Runtime;
pub use crate::network::circuit_breaker::{CircuitBreaker, CircuitState};
pub use crate::sharding::hilbert::HilbertCurve;
pub use crate::sharding::vector_index::{DistanceMetric, SearchResult, VectorIndex};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Health check function
pub fn health_check() -> bool {
    true
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
