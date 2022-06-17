pub mod jsonrpc;

pub trait NamedService {
    const SERVICE_NAME: &'static str;
}
