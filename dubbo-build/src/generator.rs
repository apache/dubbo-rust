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

use prost_build::{Method, Service, ServiceGenerator};

#[derive(Default)]
pub struct CodeGenerator {}

impl CodeGenerator {
    pub fn new() -> CodeGenerator { Default::default() }

    fn generate_module_name(&self) -> &str { "xds" }

    fn generate_uses(&mut self, buf: &mut String) {
        buf.push_str(&format!(
            "\n\
                use async_trait::async_trait;\n\
                use tower::Service;\n\
                use hyper::{{\n    \
                    Body,\n    \
                    Request,\n    \
                    Response,\n\
                }};\n\
                use futures::future::{{\n    \
                    BoxFuture\n\
                }};\n\
                use std::{{\n    \
                    task::{{Poll}},\n\
                }};\n"));
    }

    fn generate_type_aliases(&mut self, buf: &mut String) {
        buf.push_str(&format!(
            "\npub type DBResp<O> = Result<{0}::response::ServiceResponse<O>, {0}::error::DBProstError>;\n",
            self.generate_module_name()));
    }

    fn generate_main_trait(&self, service: &Service, buf: &mut String) {
        buf.push_str("\n\n#[async_trait]\n");
        service.comments.append_with_indent(0, buf);
        buf.push_str(&format!("pub trait {} {{", service.name));
        for method in service.methods.iter() {
            buf.push_str("\n");
            method.comments.append_with_indent(1, buf);
            buf.push_str(&format!("   {};\n", self.method_sig(method)));
        }
        buf.push_str("}\n");
    }

    fn method_sig(&self, method: &Method) -> String {
        format!("async fn {0}(&self, request: {1}) -> DBResp<{2}>",
                method.name, method.input_type, method.output_type)
    }

    fn generate_client_struct(&self, service: &Service, buf: &mut String) {
        buf.push_str(&format!(
            "\n\
            pub struct {0}Client {{\n    \
                pub rpc_client: {1}::client::RpcClient \n\
            }}\n",service.name, self.generate_module_name()));
    }

    fn generate_client_impl(&self, service: &Service, buf: &mut String) {
        buf.push_str(&format!(
            "\n\
            impl {0}Client {{\n     \
               pub fn new(addr: String) -> {0}Client {{\n        \
                   {0}Client {{\n            \
                        rpc_client: {1}::client::RpcClient::new(addr) \n        \
                   }}\n     \
               }}\n\
            }}\n", service.name, self.generate_module_name()));

        buf.push_str("\n#[async_trait]");
        buf.push_str(&format!("\nimpl {0} for {0}Client {{", service.name));

        for method in service.methods.iter() {
            buf.push_str(&format!(
                "\n    {0} {{\n        \
                    let path = \"dubbo/{1}.{2}/{3}\".to_owned();\n        \
                    self.rpc_client.request(request, path).await\n    \
                }}\n", self.method_sig(method), service.package, service.proto_name, method.proto_name));
        }
        buf.push_str("}\n");
    }

    fn generate_server_struct(&self, service: &Service, buf: &mut String) {
        buf.push_str(&format!(
            "\n\
            pub struct {0}Server<T: 'static + {0} + Send + Sync + Clone> {{\n    \
                pub inner: T\n\
            }}\n",service.name));
    }

    fn generate_server_impl(&self, service: &Service, buf: &mut String) {
        buf.push_str(&format!(
            "\n\
             impl<T: 'static + {0} + Send + Sync + Clone> {0}Server<T> {{\n    \
                pub fn new(service: T) -> {0}Server<T> {{\n        \
                    {0}Server {{\n            \
                        inner: service \n        \
                   }}\n     \
                }}\n\
            }}\n",
            service.name));

        buf.push_str(&format!(
            "\n\
            impl<T> Service<Request<Body>> for {0}Server<T> \n    \
                where T: 'static + Greeter + Send + Sync + Clone {{\n    \
                type Response = Response<Body>;\n    \
                type Error = hyper::Error;\n    \
                type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;\n    \
                \n    \

                fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {{\n        \
                    Poll::Ready(Ok(()))\n    \
                }}\n    \

                \n    \

                fn call(&mut self, req: Request<Body>) -> Self::Future {{\n       \
                    let inner = self.inner.clone();\n       \
                    let url = req.uri().path();\n       \
                    match (req.method().clone(), url) {{",
            service.name));

        // Make match arms for each type
        for method in service.methods.iter() {
            buf.push_str(&format!(
                "\n          \
                (::hyper::Method::POST, \"/dubbo/{0}.{1}/{2}\") => {{\n              \
                    Box::pin(async move {{\n                  \
                        let request = {3}::ServiceRequest::try_from_hyper(req).await;\n                  \
                        let proto_req = request.unwrap().try_decode().unwrap();\n                  \
                        let resp = inner.{4}(proto_req.input).await.unwrap();\n                  \
                        let proto_resp = resp.try_encode();\n                  \
                        let hyper_resp = proto_resp.unwrap().into_hyper();\n                  \
                        Ok(hyper_resp)  \n               \
                    }}) \n           \
                 }},", service.package, service.proto_name, method.proto_name, self.generate_module_name(), method.name));
        }
        // Final 404 arm and end fn
        buf.push_str(&format!(
            "\n          \
                _ => {{\n            \
                    Box::pin(async move {{ \n                \
                        Ok({0}::error::DBError::new(::hyper::StatusCode::NOT_FOUND, \"not_found\", \"Not found\").to_hyper_resp()) \n            \
                    }}) \n          \
                }}\n       \
            }}\n   \
            }}\n\
        }}", self.generate_module_name()));
    }
}

impl ServiceGenerator for CodeGenerator {
    fn generate(&mut self, service: Service, buf: &mut String) {
        self.generate_uses(buf);
        self.generate_type_aliases(buf);
        self.generate_main_trait(&service, buf);
        self.generate_client_struct(&service, buf);
        self.generate_client_impl(&service, buf);
        self.generate_server_struct(&service, buf);
        self.generate_server_impl(&service, buf);
    }

    fn finalize(&mut self, buf: &mut String) {
    }
}
