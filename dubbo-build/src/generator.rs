use prost_build::{Method, Service, ServiceGenerator};

#[derive(Default)]
pub struct CodeGenerator {}

impl CodeGenerator {
    pub fn new() -> CodeGenerator { Default::default() }

    fn generate_module_name(&self) -> &str { "xds" }

    fn generate_type_aliases(&mut self, buf: &mut String) {
        buf.push_str(&format!(
            "\n\
                pub type DBReq<I> = {0}::request::ServiceRequest<I>;\n\
                pub type DBResp<O> = Result<{0}::response::ServiceResponse<O>, {0}::error::DBProstError>;\n",
            self.generate_module_name()));
    }

    fn generate_main_trait(&self, service: &Service, buf: &mut String) {
        buf.push_str("\nuse async_trait::async_trait;\n\n#[async_trait]\n");

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
        format!("async fn {0}(&self, request: DBReq<{1}>) -> DBResp<{2}>",
                method.name, method.input_type, method.output_type)
    }

    fn generate_client_struct(&self, service: &Service, buf: &mut String) {
        buf.push_str(&format!(
            "\n\
            pub struct {0}Client {{\n    \
                pub hyper_client: {1}::wrapper::HyperClient \n\
            }}\n",service.name, self.generate_module_name()));
    }

    fn generate_client_impl(&self, service: &Service, buf: &mut String) {
        buf.push_str(&format!(
            "\n\
            impl {0}Client {{\n     \
               pub fn new(root_url: &str) -> {0}Client {{\n        \
                   {0}Client {{\n            \
                        hyper_client: {1}::wrapper::HyperClient::new(root_url) \n        \
                   }}\n     \
               }}\n\
            }}\n", service.name, self.generate_module_name()));

        buf.push_str("\n#[async_trait]");
        buf.push_str(&format!("\nimpl {0} for {0}Client {{", service.name));

        for method in service.methods.iter() {
            buf.push_str(&format!(
                "\n    {} {{\n        \
                    self.hyper_client.request(\"/dubbo/{}.{}/{}\", request).await\n    \
                }}\n", self.method_sig(method), service.package, service.proto_name, method.proto_name));
        }
        buf.push_str("}\n");
    }

    fn generate_server_struct(&self, service: &Service, buf: &mut String) {
        buf.push_str(&format!(
            "\n\
            pub struct {0}Server<T: 'static + {0} + Send + Sync + Clone> {{\n    \
                pub hyper_server: {1}::wrapper::HyperServer<T> \n\
            }}\n",service.name, self.generate_module_name()));
    }

    fn generate_server_impl(&self, service: &Service, buf: &mut String) {
        buf.push_str(&format!(
            "\n\
             impl<T: 'static + {0} + Send + Sync + Clone> {0}Server<T> {{\n    \
                pub fn new(&self, service: T) -> {0}Server<T> {{\n        \
                    {0}Server {{\n            \
                        hyper_server: {1}::wrapper::HyperServer::new(service) \n        \
                   }}\n     \
                }}\n\
            }}\n",
            service.name, self.generate_module_name()));

        buf.push_str(&format!(
            "\n\
            impl<T: 'static + {0} + Send + Sync + Clone> {1}::wrapper::HyperService for {0}Server<T> {{\n    \
                fn handle(&self, req: DBReq<Vec<u8>>) -> {1}::BoxFutureResp<Vec<u8>> {{\n       \
                    use ::futures::Future;\n       \
                    let trait_object_service = self.hyper_server.service.clone();\n       \
                    match (req.method.clone(), req.uri.path()) {{",
            service.name, self.generate_module_name()));

        // Make match arms for each type
        for method in service.methods.iter() {
            buf.push_str(&format!(
                "\n          \
                (::hyper::Method::POST, \"/dubbo/{0}.{1}/{2}\") => {{\n              \
                    Box::pin(async move {{ \n                  \
                        let proto_req = req.to_proto().unwrap();  \n                  \
                        let resp = trait_object_service.{3}(proto_req).await.unwrap();  \n                  \
                        let proto_resp = resp.to_proto_raw();  \n                  \
                        proto_resp  \n               \
                    }}) \n           \
                 }},", service.package, service.proto_name, method.proto_name, method.name));
        }
        // Final 404 arm and end fn
        buf.push_str(&format!(
            "\n          \
                _ => {{\n            \
                    Box::pin(async move {{ \n                \
                        Ok({0}::error::DBError::new(::hyper::StatusCode::NOT_FOUND, \"not_found\", \"Not found\").to_resp_raw()) \n            \
                    }}) \n          \
                }}\n       \
            }}\n   \
            }}\n\
        }}", self.generate_module_name()));
    }
}

impl ServiceGenerator for CodeGenerator {
    fn generate(&mut self, service: Service, buf: &mut String) {
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
