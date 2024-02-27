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

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::{
    self, parse_macro_input, FnArg, ImplItem, ItemImpl, ItemTrait, ReturnType, Token, TraitItem,
};

#[proc_macro_attribute]
pub fn rpc_trait(attr: TokenStream, item: TokenStream) -> TokenStream {
    let (package, version) = parse_attr(attr);
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
        fn_quote.push(
            quote! {
                    #[allow(non_snake_case)]
                    pub #asyncable fn #ident (#(#inputs),*) -> Result<#output_type,dubbo::status::Status> {
                    let mut req_vec : Vec<String> = vec![];
                    #(
                        let mut res_str = serde_json::to_string(&#req);
                        if let Err(err) = res_str {
                            return Err(dubbo::status::Status::new(dubbo::status::Code::InvalidArgument,err.to_string()));
                        }
                        req_vec.push(res_str.unwrap());
                    )*
                    let _version : Option<&str> = #version;
                    let request = Request::new(TripleRequestWrapper::new(req_vec));
                    let codec = ProstCodec::<
                     TripleRequestWrapper,
                     TripleResponseWrapper
                    >::default();
                    let service_unique = #package.to_owned() + "." + stringify!(#trait_ident);
                    let method_name = stringify!(#ident).to_string();
                    let invocation = dubbo::invocation::RpcInvocation::default()
                                    .with_service_unique_name(service_unique.clone())
                                    .with_method_name(method_name.clone());
                    let path = "/".to_string() + &service_unique + "/" + &method_name;
                    let path = http::uri::PathAndQuery::from_str(
                         &path,
                    ).unwrap();
                    let res = self.inner.unary(request, codec, path, invocation).await;
                    match res {
                        Ok(res) => {
                            let response_wrapper = res.into_parts().1;
                            let res: #output_type = serde_json::from_slice(&response_wrapper.data).unwrap();
                             Ok(res)
                        },
                        Err(err) => Err(err)
                    }
                }
            }
        );
    }
    let rpc_client = syn::Ident::new(&format!("{}Rpc", trait_ident), trait_ident.span());
    let expanded = quote! {
        use dubbo::triple::client::TripleClient;
        use dubbo::triple::triple_wrapper::TripleRequestWrapper;
        use dubbo::triple::triple_wrapper::TripleResponseWrapper;
        use dubbo::triple::codec::prost::ProstCodec;
        use dubbo::invocation::Request;
        use dubbo::invocation::Response;
        use dubbo::triple::client::builder::ClientBuilder;
        use std::str::FromStr;

        #item_trait

        #vis struct #rpc_client {
            inner: TripleClient
        }
        impl #rpc_client {
            #(
                #fn_quote
            )*
            pub fn new(builder: ClientBuilder) -> #rpc_client {
                #rpc_client {inner: TripleClient::new(builder),}
            }
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn rpc_server(attr: TokenStream, item: TokenStream) -> TokenStream {
    let (package, version) = parse_attr(attr);
    let org_item = parse_macro_input!(item as ItemImpl);
    let server_item = get_server_item(org_item.clone());
    let item = org_item.clone();
    let item_trait = &item.trait_.unwrap().1.segments[0].ident;
    let item_self = item.self_ty;
    let items_ident_fn = item.items.iter().fold(vec![], |mut vec, e| {
        if let ImplItem::Fn(fn_item) = e {
            vec.push(fn_item.sig.ident.clone())
        }
        vec
    });
    let items_fn = item.items.iter().fold(vec![], |mut vec, e| {
        if let ImplItem::Fn(fn_item) = e {
            let method = &fn_item.sig.ident;
            let mut req_pat = vec![];
            let req = fn_item.sig.inputs.iter().fold(vec![], |mut vec, e| {
                if let FnArg::Typed(input) = e {
                    let req = &input.pat;
                    let req_type = &input.ty;
                    let token = quote! {
                     let result : Result<#req_type,_>  = serde_json::from_slice(param_req[idx].as_bytes());
                    if let Err(err) = result {
                        param.res = Err(dubbo::status::Status::new(dubbo::status::Code::InvalidArgument,err.to_string()));
                        return param;
                    }
                    let #req : #req_type = result.unwrap();
                    idx += 1;
                    };
                    req_pat.push(req);
                    vec.push(token);
                }
                vec
            },
            );
            vec.push(quote! {
                if &param.method_name[..] == stringify!(#method) {
                let param_req = &param.req;
                let mut idx = 0;
                #(
                    #req
                )*
                let res = self.#method(
                    #(
                        #req_pat,
                    )*
                ).await;
                param.res = match res {
                    Ok(res) => {
                        let res = serde_json::to_string(&res).unwrap();
                        Ok(res)
                    },
                    Err(info) => Err(info)
                };
                return param;
            }
            }
            )
        }
        vec
    });
    let expanded = quote! {
        #server_item
        use dubbo::triple::server::support::RpcServer;
        use dubbo::triple::server::support::RpcFuture;
        use dubbo::triple::server::support::RpcMsg;

        impl RpcServer for #item_self {
            fn invoke (&self, param : RpcMsg) -> RpcFuture<RpcMsg> {
                let mut rpc = self.clone();
                Box::pin(async move {rpc.prv_invoke(param).await})
            }
            fn get_info(&self) -> (&str , &str , Option<&str> , Vec<String>) {
               let mut methods = vec![];
               #(
                  methods.push(stringify!(#items_ident_fn).to_string());
               )*
               (#package ,stringify!(#item_trait) , #version ,methods)
            }
        }

        impl #item_self {
            async fn prv_invoke (&self, mut param : RpcMsg) -> RpcMsg {
                #(#items_fn)*
                param.res = Err(
                    dubbo::status::Status::new(dubbo::status::Code::NotFound,format!("not find method by {}",param.method_name))
                );
                return param;
            }
        }
    };
    expanded.into()
}

fn parse_attr(attr: TokenStream) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let mut map = HashMap::new();
    let attr = attr.clone().to_string();
    let args: Vec<&str> = attr.split(",").collect();
    for arg in args {
        let arg = arg.replace(" ", "");
        let item: Vec<&str> = arg.split("=").collect();
        map.insert(
            item[0].to_string().clone(),
            item[1].replace("\"", "").to_string().clone(),
        );
    }
    let package = map.get("package").map_or("krpc", |e| e);
    let package = quote!(#package);
    let version = match map.get("version").map(|e| e.to_string()) {
        None => quote!(None),
        Some(version) => quote!(Some(&#version)),
    };
    return (package, version);
}

fn get_server_item(item: ItemImpl) -> proc_macro2::TokenStream {
    let impl_item = item.impl_token;
    let trait_ident = item.trait_.unwrap().1;
    let ident = item.self_ty.to_token_stream();
    let fn_items = item.items.iter().fold(vec![], |mut vec, e| {
        if let ImplItem::Fn(fn_item) = e {
            vec.push(fn_item);
        }
        vec
    });
    quote! {
        #impl_item #trait_ident for #ident {
            #(
                #[allow(non_snake_case)]
                #fn_items
            )*
        }
    }
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
