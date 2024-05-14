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
use syn::{parse_macro_input, FnArg, ImplItem, ItemImpl};

pub fn dubbo_server(attr: DubboAttr, item: TokenStream) -> TokenStream {
    let version = match attr.version {
        Some(version) => quote!(Some(&#version)),
        None => quote!(None),
    };
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
                        param.result = Err(dubbo::status::Status::new(dubbo::status::Code::InvalidArgument,err.to_string()));
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
                let param_req = &param.args;
                let mut idx = 0;
                #(
                    #req
                )*
                let res = self.#method(
                    #(
                        #req_pat,
                    )*
                ).await;
                param.result = match res {
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
    let service_unique = match &attr.package {
        None => {
            quote!(stringify!(#item_trait))
        }
        Some(attr) => {
            let service_unique = attr.to_owned() + "." + &item_trait.to_string();
            quote!(&#service_unique)
        }
    };
    let expanded = quote! {

        #server_item
        impl dubbo::triple::server::support::RpcServer for #item_self {
            fn invoke (&self, param : dubbo::triple::server::support::RpcContext) -> dubbo::triple::server::support::RpcFuture<dubbo::triple::server::support::RpcContext> {
                let mut rpc = self.clone();
                Box::pin(async move {rpc.prv_invoke(param).await})
            }
            fn get_info(&self) -> (&str, Option<&str>, Vec<String>) {
               let mut methods = vec![];
               #(
                  methods.push(stringify!(#items_ident_fn).to_string());
               )*
               (#service_unique , #version ,methods)
            }
        }

        impl #item_self {
            async fn prv_invoke (&self, mut param : dubbo::triple::server::support::RpcContext) -> dubbo::triple::server::support::RpcContext {
                #(#items_fn)*
                param.result = Err(
                    dubbo::status::Status::new(dubbo::status::Code::NotFound,format!("not find method by {}",param.method_name))
                );
                return param;
            }
        }
    };
    expanded.into()
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
