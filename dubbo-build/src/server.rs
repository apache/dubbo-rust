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

use super::{generate_doc_comment, generate_doc_comments, naive_snake_case, Attributes};
use crate::{Method, Service};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, Lit, LitStr};

/// Generate service for Server.
///
/// This takes some `Service` and will generate a `TokenStream` that contains
/// a public module containing the server service and handler trait.
pub fn generate<T: Service>(
    service: &T,
    emit_package: bool,
    proto_path: &str,
    compile_well_known_types: bool,
    attributes: &Attributes,
) -> TokenStream {
    let methods = generate_methods(service, proto_path, compile_well_known_types);

    let server_service = quote::format_ident!("{}Server", service.name());
    let server_trait = quote::format_ident!("{}", service.name());
    let server_mod = quote::format_ident!("{}_server", naive_snake_case(service.name()));
    let generated_trait = generate_trait(
        service,
        proto_path,
        compile_well_known_types,
        server_trait.clone(),
    );
    let service_doc = generate_doc_comments(service.comment());
    let package = if emit_package { service.package() } else { "" };
    // Transport based implementations
    let path = format!(
        "{}{}{}",
        package,
        if package.is_empty() { "" } else { "." },
        service.identifier()
    );
    let service_name = syn::LitStr::new(&path, proc_macro2::Span::call_site());
    let mod_attributes = attributes.for_mod(package);
    let struct_attributes = attributes.for_struct(&path);

    quote! {
        /// Generated server implementations.
        #(#mod_attributes)*
        pub mod #server_mod {
            #![allow(
                unused_variables,
                dead_code,
                missing_docs,
                // will trigger if compression is disabled
                clippy::let_unit_value,
            )]
            use dubbo::codegen::*;

            #generated_trait

            #service_doc
            #(#struct_attributes)*
            #[derive(Debug)]
            pub struct #server_service<T: #server_trait> {
                inner: _Inner<T>,
            }

            struct _Inner<T>(Arc<T>);

            impl<T: #server_trait> #server_service<T> {
                pub fn new(inner: T) -> Self {
                    Self {
                        inner: _Inner(Arc::new(inner)),
                    }
                }

                pub fn with_filter<F>(inner: T, filter: F) -> FilterService<Self, F>
                where
                    F: Filter,
                {
                    FilterService::new(Self::new(inner), filter)
                }

            }

            impl<T, B> Service<http::Request<B>> for #server_service<T>
                where
                    T: #server_trait,
                    B: Body + Send + 'static,
                    B::Error: Into<StdError> + Send + 'static,
            {
                type Response = http::Response<BoxBody>;
                type Error = std::convert::Infallible;
                type Future = BoxFuture<Self::Response, Self::Error>;

                fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                    Poll::Ready(Ok(()))
                }

                fn call(&mut self, req: http::Request<B>) -> Self::Future {
                    let inner = self.inner.clone();

                    match req.uri().path() {
                        #methods

                        _ => Box::pin(async move {
                            Ok(http::Response::builder()
                               .status(200)
                               .header("grpc-status", "12")
                               .header("content-type", "application/grpc")
                               .body(empty_body())
                               .unwrap())
                        }),
                    }
                }
            }

            impl<T: #server_trait> Clone for #server_service<T> {
                fn clone(&self) -> Self {
                    let inner = self.inner.clone();
                    Self {
                        inner,
                    }
                }
            }

            impl<T: #server_trait> Clone for _Inner<T> {
                fn clone(&self) -> Self {
                    Self(self.0.clone())
                }
            }

            impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                   write!(f, "{:?}", self.0)
                }
            }

            pub fn register_server<T: #server_trait>(server: T) {
                let s = #server_service::new(server);
                dubbo::protocol::triple::TRIPLE_SERVICES
                    .write()
                    .unwrap()
                    .insert(
                        #service_name.to_string(),
                        dubbo::utils::boxed_clone::BoxCloneService::new(s),
                    );
            }
        }
    }
}

fn generate_trait<T: Service>(
    service: &T,
    proto_path: &str,
    compile_well_known_types: bool,
    server_trait: Ident,
) -> TokenStream {
    let methods = generate_trait_methods(service, proto_path, compile_well_known_types);
    let trait_doc = generate_doc_comment(&format!(
        "Generated trait containing gRPC methods that should be implemented for use with {}Server.",
        service.name()
    ));

    quote! {
        #trait_doc
        #[async_trait]
        pub trait #server_trait : Send + Sync + 'static {
            #methods
        }
    }
}

fn generate_trait_methods<T: Service>(
    service: &T,
    proto_path: &str,
    compile_well_known_types: bool,
) -> TokenStream {
    let mut stream = TokenStream::new();

    for method in service.methods() {
        let name = quote::format_ident!("{}", method.name());

        let (req_message, res_message) =
            method.request_response_name(proto_path, compile_well_known_types);

        let method_doc = generate_doc_comments(method.comment());

        let method = match (method.client_streaming(), method.server_streaming()) {
            (false, false) => {
                quote! {
                    #method_doc
                    async fn #name(&self, request: Request<#req_message>)
                        -> Result<Response<#res_message>, dubbo::status::Status>;
                }
            }
            (true, false) => {
                quote! {
                    #method_doc
                    async fn #name(&self, request: Request<Decoding<#req_message>>)
                        -> Result<Response<#res_message>, dubbo::status::Status>;
                }
            }
            (false, true) => {
                let stream = quote::format_ident!("{}Stream", method.identifier());
                let stream_doc = generate_doc_comment(&format!(
                    "Server streaming response type for the {} method.",
                    method.identifier()
                ));

                quote! {
                    #stream_doc
                    type #stream: futures_util::Stream<Item = Result<#res_message, dubbo::status::Status>> + Send + 'static;

                    #method_doc
                    async fn #name(&self, request: Request<#req_message>)
                        -> Result<Response<Self::#stream>, dubbo::status::Status>;
                }
            }
            (true, true) => {
                let stream = quote::format_ident!("{}Stream", method.identifier());
                let stream_doc = generate_doc_comment(&format!(
                    "Server streaming response type for the {} method.",
                    method.identifier()
                ));

                quote! {
                    #stream_doc
                    type #stream: futures_util::Stream<Item = Result<#res_message, dubbo::status::Status>> + Send + 'static;

                    #method_doc
                    async fn #name(&self, request: Request<Decoding<#req_message>>)
                        -> Result<Response<Self::#stream>, dubbo::status::Status>;
                }
            }
        };

        stream.extend(method);
    }

    stream
}

fn generate_methods<T: Service>(
    service: &T,
    proto_path: &str,
    compile_well_known_types: bool,
) -> TokenStream {
    let mut stream = TokenStream::new();

    for method in service.methods() {
        let path = format!(
            "/{}{}{}/{}",
            service.package(),
            if service.package().is_empty() {
                ""
            } else {
                "."
            },
            service.identifier(),
            method.identifier()
        );
        let method_path = Lit::Str(LitStr::new(&path, Span::call_site()));
        let ident = quote::format_ident!("{}", method.name());
        let server_trait = quote::format_ident!("{}", service.name());

        let method_stream = match (method.client_streaming(), method.server_streaming()) {
            (false, false) => generate_unary(
                &method,
                proto_path,
                compile_well_known_types,
                ident,
                server_trait,
            ),

            (false, true) => generate_server_streaming(
                &method,
                proto_path,
                compile_well_known_types,
                ident.clone(),
                server_trait,
            ),
            (true, false) => generate_client_streaming(
                &method,
                proto_path,
                compile_well_known_types,
                ident.clone(),
                server_trait,
            ),

            (true, true) => generate_streaming(
                &method,
                proto_path,
                compile_well_known_types,
                ident.clone(),
                server_trait,
            ),
        };

        let method = quote! {
            #method_path => {
                #method_stream
            }
        };
        stream.extend(method);
    }

    stream
}

fn generate_unary<T: Method>(
    method: &T,
    proto_path: &str,
    compile_well_known_types: bool,
    method_ident: Ident,
    server_trait: Ident,
) -> TokenStream {
    let service_ident = quote::format_ident!("{}Server", method.identifier());

    let (request, response) = method.request_response_name(proto_path, compile_well_known_types);

    quote! {
        #[allow(non_camel_case_types)]
        struct #service_ident<T: #server_trait > {
            inner: _Inner<T>,
        };

        impl<T: #server_trait> UnarySvc<#request> for #service_ident<T> {
            type Response = #response;
            type Future = BoxFuture<Response<Self::Response>, dubbo::status::Status>;

            fn call(&mut self, request: Request<#request>) -> Self::Future {
                let inner = self.inner.0.clone();
                let fut = async move {
                    inner.#method_ident(request).await
                };
                Box::pin(fut)
            }
        }
        let fut = async move {
            let mut server = TripleServer::<#request,#response>::new();
            let res = server.unary(#service_ident { inner }, req).await;
            Ok(res)
        };
        Box::pin(fut)
    }
}

fn generate_server_streaming<T: Method>(
    method: &T,
    proto_path: &str,
    compile_well_known_types: bool,
    method_ident: Ident,
    server_trait: Ident,
) -> TokenStream {
    let service_ident = quote::format_ident!("{}Server", method.identifier());

    let (request, response) = method.request_response_name(proto_path, compile_well_known_types);

    let response_stream = quote::format_ident!("{}Stream", method.identifier());

    quote! {
        #[allow(non_camel_case_types)]
        struct #service_ident<T: #server_trait > {
            inner: _Inner<T>,
        };

        impl<T: #server_trait> ServerStreamingSvc<#request> for #service_ident<T> {
            type Response = #response;
            type ResponseStream = T::#response_stream;
            type Future = BoxFuture<Response<Self::ResponseStream>, dubbo::status::Status>;

            fn call(&mut self, request: Request<#request>) -> Self::Future {
                let inner = self.inner.0.clone();
                let fut = async move {
                    inner.#method_ident(request).await
                };
                Box::pin(fut)
            }
        }
        let fut = async move {
            let mut server = TripleServer::<#request,#response>::new();
            let res = server.server_streaming(#service_ident { inner }, req).await;
            Ok(res)
        };

        Box::pin(fut)
    }
}

fn generate_client_streaming<T: Method>(
    method: &T,
    proto_path: &str,
    compile_well_known_types: bool,
    method_ident: Ident,
    server_trait: Ident,
) -> TokenStream {
    let service_ident = quote::format_ident!("{}Server", method.identifier());

    let (request, response) = method.request_response_name(proto_path, compile_well_known_types);

    quote! {
        #[allow(non_camel_case_types)]
        struct #service_ident<T: #server_trait >{
            inner: _Inner<T>,
        };

        impl<T: #server_trait> ClientStreamingSvc<#request> for #service_ident<T>
        {
            type Response = #response;
            type Future = BoxFuture<Response<Self::Response>, dubbo::status::Status>;

            fn call(&mut self, request: Request<Decoding<#request>>) -> Self::Future {
                let inner = self.inner.0.clone();
                let fut = async move {
                    inner.#method_ident(request).await

                };
                Box::pin(fut)
            }
        }

        let fut = async move {
            let mut server = TripleServer::<#request,#response>::new();
            let res = server.client_streaming(#service_ident { inner }, req).await;
            Ok(res)
        };

        Box::pin(fut)
    }
}

fn generate_streaming<T: Method>(
    method: &T,
    proto_path: &str,
    compile_well_known_types: bool,
    method_ident: Ident,
    server_trait: Ident,
) -> TokenStream {
    let service_ident = quote::format_ident!("{}Server", method.identifier());

    let (request, response) = method.request_response_name(proto_path, compile_well_known_types);

    let response_stream = quote::format_ident!("{}Stream", method.identifier());

    quote! {
        #[allow(non_camel_case_types)]
        struct #service_ident<T: #server_trait>{
            inner: _Inner<T>,
        };

        impl<T: #server_trait> StreamingSvc<#request> for #service_ident<T>
        {
            type Response = #response;
            type ResponseStream = T::#response_stream;
            type Future = BoxFuture<Response<Self::ResponseStream>, dubbo::status::Status>;

            fn call(&mut self, request: Request<Decoding<#request>>) -> Self::Future {
                let inner = self.inner.0.clone();
                let fut = async move {
                    inner.#method_ident(request).await
                };
                Box::pin(fut)
            }
        }

        let fut = async move {
            let mut server = TripleServer::<#request,#response>::new();
            let res = server.bidi_streaming(#service_ident { inner }, req).await;
            Ok(res)
        };

        Box::pin(fut)
    }
}
