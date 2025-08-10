//! E2E tests for kernel-gossip
//! 
//! NO MOCKS - All tests use REAL Kubernetes cluster and REAL workloads

mod environment;
mod workload;

pub use environment::E2ETestEnvironment;
pub use workload::TestWorkload;
