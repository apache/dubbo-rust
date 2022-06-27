
use crate::service::protocol::*;
use crate::service::protocol::Invoker;    

pub struct GrpcExporter<T> {
    invoker: T
}

impl<T> GrpcExporter<T> {
    pub fn new(_key: String, invoker: T) -> GrpcExporter<T> {
        Self { invoker }
    }
}

impl<T: Invoker+Clone> Exporter for GrpcExporter<T>
{
    type InvokerType = T;

    fn unexport(&self) {
    }

    fn get_invoker(&self) -> Self::InvokerType {
        self.invoker.clone()
    }

}

impl<T: Invoker+Clone> Clone for GrpcExporter<T> {
    
    fn clone(&self) -> Self {
        Self { invoker: self.invoker.clone() }
    }

}