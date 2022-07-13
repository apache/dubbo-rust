use futures::future::BoxFuture;
use crate::request::ServiceRequest;
use crate::response::ServiceResponse;
use crate::error::DBProstError;


pub type BoxFutureReq<T> = BoxFuture<'static, Result<ServiceRequest<T>, DBProstError>>;
pub type BoxFutureResp<O> = BoxFuture<'static, Result<ServiceResponse<O>, DBProstError>>;

