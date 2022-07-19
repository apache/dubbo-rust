use super::triple_invoker::TripleInvoker;
use crate::protocol::Exporter;

#[derive(Clone)]
pub struct TripleExporter {}

impl TripleExporter {
    pub fn new() -> Self {
        TripleExporter {}
    }
}

impl Default for TripleExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Exporter for TripleExporter {
    type InvokerType = TripleInvoker;

    fn unexport(&self) {
        todo!()
    }

    fn get_invoker(&self) -> Self::InvokerType {
        todo!()
    }
}
