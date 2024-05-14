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
use quote::ToTokens;
use syn::parse::Parser;

mod server_macro;
mod trait_macro;

#[proc_macro_attribute]
pub fn dubbo_trait(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = DubboAttr::from_attr(attr);
    match attr {
        Ok(attr) => trait_macro::dubbo_trait(attr, item),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn dubbo_server(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = DubboAttr::from_attr(attr);
    match attr {
        Ok(attr) => server_macro::dubbo_server(attr, item),
        Err(err) => err.into_compile_error().into(),
    }
}

#[derive(Default)]
struct DubboAttr {
    package: Option<String>,
    version: Option<String>,
}

impl DubboAttr {
    fn from_attr(args: TokenStream) -> Result<DubboAttr, syn::Error> {
        syn::punctuated::Punctuated::<syn::Meta, syn::Token![, ]>::parse_terminated
            .parse2(args.into())
            .and_then(|args| Self::build_attr(args))
    }

    fn build_attr(
        args: syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>,
    ) -> Result<DubboAttr, syn::Error> {
        let mut package = None;
        let mut version = None;
        for arg in args {
            match arg {
                syn::Meta::NameValue(namevalue) => {
                    let ident = namevalue
                        .path
                        .get_ident()
                        .ok_or_else(|| {
                            syn::Error::new_spanned(&namevalue, "Must have specified ident")
                        })?
                        .to_string()
                        .to_lowercase();
                    let lit = match &namevalue.value {
                        syn::Expr::Lit(syn::ExprLit { lit, .. }) => {
                            lit.to_token_stream().to_string()
                        }
                        expr => expr.to_token_stream().to_string(),
                    }
                    .replace("\"", "");
                    match ident.as_str() {
                        "package" => {
                            let _ = package.insert(lit);
                        }
                        "version" => {
                            let _ = version.insert(lit);
                        }
                        name => {
                            let msg = format!(
                                "Unknown attribute {} is specified; expected one of: {} ",
                                name, "'package','version'",
                            );
                            return Err(syn::Error::new_spanned(namevalue, msg));
                        }
                    }
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "Unknown attribute inside the dubbo-macro",
                    ));
                }
            }
        }
        Ok(DubboAttr { package, version })
    }
}
