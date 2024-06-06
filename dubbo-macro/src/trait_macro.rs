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

use crate::DubboAttr;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, FnArg, ItemTrait, ReturnType, TraitItem};

pub fn dubbo_trait(attr: DubboAttr, item: TokenStream) -> TokenStream {
    let version = match attr.version {
        Some(version) => quote!(Some(&#version)),
        None => quote!(None),
    };
    let input = parse_macro_input!(item as ItemTrait);
    let item_trait = get_item_trait(input.clone());
    let trait_ident = &input.ident;
    let vis = &input.vis;
    let items = &input.items;
    let mut sig_item = vec![];
    for item in items {
        if let TraitItem::Fn(item) = item {
            sig_item.push(item.sig.clone());
        }
    }
    let mut fn_quote = vec![];
    for item in sig_item {
        let asyncable = item.asyncness;
        let ident = item.ident;
        let inputs = item.inputs;
        let req = inputs.iter().fold(vec![], |mut vec, e| {
            if let FnArg::Typed(req) = e {
                vec.push(req.pat.clone());
            }
            vec
        });
        let output = item.output;
        let output_type = match &output {
            ReturnType::Default => {
                quote! {()}
            }
            ReturnType::Type(_, res_type) => res_type.to_token_stream(),
        };
        let inputs = inputs.iter().fold(vec![], |mut vec, e| {
            let mut token = e.to_token_stream();
            if vec.is_empty() {
                if let FnArg::Receiver(_r) = e {
                    token = quote!(&mut self);
                }
            }
            vec.push(token);
            vec
        });
        let package = trait_ident.to_string();
        let service_unique = match &attr.package {
            None => package.to_owned(),
            Some(attr) => attr.to_owned() + "." + &package,
        };
        let path = "/".to_string() + &service_unique + "/" + &ident.to_string();
        fn_quote.push(
            quote! {
                    #[allow(non_snake_case)]
                    pub #asyncable fn #ident (#(#inputs),*) -> Result<#output_type,dubbo::status::Status> {
                    let mut req_vec : Vec<String> = vec![];
                    #(
                        let mut req_poi_str = serde_json::to_string(&#req);
                        if let Err(err) = req_poi_str {
                            return Err(dubbo::status::Status::new(dubbo::status::Code::InvalidArgument,err.to_string()));
                        }
                        req_vec.push(req_poi_str.unwrap());
                    )*
                    let _version : Option<&str> = #version;
                    let request = dubbo::invocation::Request::new(dubbo::triple::triple_wrapper::TripleRequestWrapper::new(req_vec));
                    let service_unique = #service_unique;
                    let method_name = stringify!(#ident).to_string();
                    let invocation = dubbo::invocation::RpcInvocation::default()
                                    .with_service_unique_name(service_unique.to_owned())
                                    .with_method_name(method_name.clone());
                    let path = http::uri::PathAndQuery::from_static(
                         #path,
                    );
                    let res = self.inner.unary::<dubbo::triple::triple_wrapper::TripleRequestWrapper,dubbo::triple::triple_wrapper::TripleResponseWrapper>(request, path, invocation).await;
                    match res {
                        Ok(res) => {
                            let response_wrapper = res.into_parts().1;
                            let data = &response_wrapper.data;
                            if data.starts_with(b"null") {
                                Err(dubbo::status::Status::new(dubbo::status::Code::DataLoss,"null".to_string()))
                            } else {
                                let res: #output_type = serde_json::from_slice(data).unwrap();
                                Ok(res)
                            }
                        },
                        Err(err) => Err(err)
                    }
                }
            }
        );
    }
    let rpc_client = syn::Ident::new(&format!("{}Client", trait_ident), trait_ident.span());
    let expanded = quote! {

        #item_trait

        #vis struct #rpc_client {
            inner: dubbo::triple::client::TripleClient
        }
        impl #rpc_client {
            #(
                #fn_quote
            )*
            pub fn new(builder: dubbo::triple::client::builder::ClientBuilder) -> #rpc_client {
                #rpc_client {inner: dubbo::triple::client::TripleClient::new(builder),}
            }
        }
    };
    TokenStream::from(expanded)
}

fn get_item_trait(item: ItemTrait) -> proc_macro2::TokenStream {
    let trait_ident = &item.ident;
    let item_fn = item.items.iter().fold(vec![], |mut vec, e| {
        if let TraitItem::Fn(item_fn) = e {
            let asyncable = &item_fn.sig.asyncness;
            let ident = &item_fn.sig.ident;
            let inputs = &item_fn.sig.inputs;
            let output_type = match &item_fn.sig.output {
                ReturnType::Default => {
                    quote! {()}
                }
                ReturnType::Type(_, res_type) => res_type.to_token_stream(),
            };
            vec.push(quote!(
               #asyncable fn #ident (#inputs) -> Result<#output_type,dubbo::status::Status>;
            ));
        }
        vec
    });
    quote! {
        pub trait #trait_ident {
           #(
               #[allow(async_fn_in_trait)]
               #[allow(non_snake_case)]
               #item_fn
            )*
        }
    }
}
