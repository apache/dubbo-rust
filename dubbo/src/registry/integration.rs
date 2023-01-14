use crate::cluster::support::cluster_invoker::ClusterInvoker;
use crate::registry::BoxRegistry;

pub trait ClusterRegistryIntegration {
    /// get cluster invoker struct
    fn get_invoker(registry: BoxRegistry) -> Option<&ClusterInvoker>;
}
