/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
//! Filters which can preprocess or postprocess request.  
//!
//! TODO: Add the `ClusterFilter` according to the name.  
//!
//! # Example
//! ## ClusterFilter
//! ```no_run
//! const USER_AGENT: &str = "user-agent";
//! const USER_AGENT_VAL: &str = "dubbo-test";
//! const USER_AGENT_VAL_2: &str = "dubbo-test-2";
//!
//! #[derive(Clone, Copy)]
//! struct MyFilter;
//!
//! impl ClusterFilter for MyFilter1 {
//!     fn call(&mut self, req: Request<()>) -> Result<Request<()>, crate::status::Status> {
//!         req.metadata_mut().get_mut().insert(
//!             USER_AGENT.to_string(),
//!             USER_AGENT_VAL.to_string(),
//!         );
//!         Ok::<_, Status>(req)
//!     }
//! }
//!
//! impl ClusterFilter for MyFilter2 {
//!     fn call(&mut self, req: Request<()>) -> Result<Request<()>, crate::status::Status> {
//!         assert_eq!(
//!             req.metadata()
//!                 .get_ref()
//!                 .get(USER_AGENT)
//!                 .expect("missing user-agent."),
//!             USER_AGENT_VAL
//!         );
//!         req.metadata_mut().get_mut().insert(
//!             USER_AGENT.to_string(),
//!             USER_AGENT_VAL_2.to_string(),
//!         );
//!         Ok::<_, Status>(req)
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!    let svc = service_fn(|req: http::Request<BoxBody>| async move {
//!        assert_eq!(
//!            req.headers().get(USER_AGENT).map(|v| v.to_str().unwrap()),
//!            Some(USER_AGENT_VAL_2)
//!        );
//!        Ok::<_, Status>(http::Response::new(empty_body()))
//!    });
//!    let svc = ServiceBuilder::new()
//!        .layer(cluster_filter(MyFilter1))
//!        .layer(cluster_filter(MyFilter2))
//!        .service(svc);
//!    let req = http::Request::builder()
//!        .body(empty_body())
//!        .unwrap();
//!    svc.oneshot(req).await.unwrap();
//! }
//! ```

use crate::codegen::Request;

pub mod clusterfilter;
pub mod service;

/// TODO: Implement it.
pub trait Filter {
    fn call(&mut self, req: Request<()>) -> Result<Request<()>, crate::status::Status>;
}

/// The `ClusterFilter` can **preprocess** the HTTP Request and then pass the result to the inner
/// [`Service`].
/// `ClusterFilter` is implemented as a tower's [`Layer`], which can let us take full advantage of the
/// other `Layer` provided by [`tower-http`].
///
/// [`tower-http`]: https://docs.rs/tower-http/latest/tower_http/
/// [`Service`]: https://docs.rs/tower/latest/tower/trait.Service.html
/// [`Layer`]: https://docs.rs/tower/latest/tower/trait.Layer.html
pub trait ClusterFilter {
    fn call(&mut self, req: Request<()>) -> Result<Request<()>, crate::status::Status>;
}
