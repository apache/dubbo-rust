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

pub mod client;
pub mod prost;
pub mod server;

use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream};
use quote::TokenStreamExt;

/// Attributes that will be added to `mod` and `struct` items.
#[derive(Debug, Default, Clone)]
pub struct Attributes {
    /// `mod` attributes.
    module: Vec<(String, String)>,
    /// `struct` attributes.
    structure: Vec<(String, String)>,
}

impl Attributes {
    fn for_mod(&self, name: &str) -> Vec<syn::Attribute> {
        generate_attributes(name, &self.module)
    }

    fn for_struct(&self, name: &str) -> Vec<syn::Attribute> {
        generate_attributes(name, &self.structure)
    }

    pub fn push_mod(&mut self, pattern: impl Into<String>, attr: impl Into<String>) {
        self.module.push((pattern.into(), attr.into()));
    }

    pub fn push_struct(&mut self, pattern: impl Into<String>, attr: impl Into<String>) {
        self.structure.push((pattern.into(), attr.into()));
    }
}

// Generates attributes given a list of (`pattern`, `attribute`) pairs. If `pattern` matches `name`, `attribute` will be included.
fn generate_attributes<'a>(
    name: &str,
    attrs: impl IntoIterator<Item = &'a (String, String)>,
) -> Vec<syn::Attribute> {
    attrs
        .into_iter()
        .filter(|(matcher, _)| match_name(matcher, name))
        .flat_map(|(_, attr)| {
            // attributes cannot be parsed directly, so we pretend they're on a struct
            syn::parse_str::<syn::DeriveInput>(&format!("{}\nstruct fake;", attr))
                .unwrap()
                .attrs
        })
        .collect::<Vec<_>>()
}

// Generate a singular line of a doc comment
fn generate_doc_comment<S: AsRef<str>>(comment: S) -> TokenStream {
    let mut doc_stream = TokenStream::new();

    doc_stream.append(Ident::new("doc", Span::call_site()));
    doc_stream.append(Punct::new('=', Spacing::Alone));
    doc_stream.append(Literal::string(comment.as_ref()));

    let group = Group::new(Delimiter::Bracket, doc_stream);

    let mut stream = TokenStream::new();
    stream.append(Punct::new('#', Spacing::Alone));
    stream.append(group);
    stream
}

// Generate a larger doc comment composed of many lines of doc comments
fn generate_doc_comments<T: AsRef<str>>(comments: &[T]) -> TokenStream {
    let mut stream = TokenStream::new();

    for comment in comments {
        stream.extend(generate_doc_comment(comment));
    }

    stream
}

// Checks whether a path pattern matches a given path.
pub(crate) fn match_name(pattern: &str, path: &str) -> bool {
    if pattern.is_empty() {
        false
    } else if pattern == "." || pattern == path {
        true
    } else {
        let pattern_segments = pattern.split('.').collect::<Vec<_>>();
        let path_segments = path.split('.').collect::<Vec<_>>();

        if &pattern[..1] == "." {
            // prefix match
            if pattern_segments.len() > path_segments.len() {
                false
            } else {
                pattern_segments[..] == path_segments[..pattern_segments.len()]
            }
        // suffix match
        } else if pattern_segments.len() > path_segments.len() {
            false
        } else {
            pattern_segments[..] == path_segments[path_segments.len() - pattern_segments.len()..]
        }
    }
}

fn naive_snake_case(name: &str) -> String {
    let mut s = String::new();
    let mut it = name.chars().peekable();

    while let Some(x) = it.next() {
        s.push(x.to_ascii_lowercase());
        if let Some(y) = it.peek() {
            if y.is_uppercase() {
                s.push('_');
            }
        }
    }

    s
}

/// Service generation trait.
///
/// This trait can be implemented and consumed
/// by `client::generate` and `server::generate`
/// to allow any codegen module to generate service
/// abstractions.
pub trait Service {
    /// Comment type.
    type Comment: AsRef<str>;

    /// Method type.
    type Method: Method;

    /// Name of service.
    fn name(&self) -> &str;
    /// Package name of service.
    fn package(&self) -> &str;
    /// Identifier used to generate type name.
    fn identifier(&self) -> &str;
    /// Methods provided by service.
    fn methods(&self) -> Vec<Self::Method>;
    /// Get comments about this item.
    fn comment(&self) -> &[Self::Comment];
}

/// Method generation trait.
///
/// Each service contains a set of generic
/// `Methods`'s that will be used by codegen
/// to generate abstraction implementations for
/// the provided methods.
pub trait Method {
    /// Comment type.
    type Comment: AsRef<str>;

    /// Name of method.
    fn name(&self) -> &str;
    /// Identifier used to generate type name.
    fn identifier(&self) -> &str;
    /// Path to the codec.
    fn codec_path(&self) -> &str;
    /// Method is streamed by client.
    fn client_streaming(&self) -> bool;
    /// Method is streamed by server.
    fn server_streaming(&self) -> bool;
    /// Get comments about this item.
    fn comment(&self) -> &[Self::Comment];
    /// Type name of request and response.
    fn request_response_name(
        &self,
        proto_path: &str,
        compile_well_known_types: bool,
    ) -> (TokenStream, TokenStream);
}
