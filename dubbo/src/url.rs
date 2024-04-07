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

use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    str::FromStr,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Url {
    inner: url::Url,
}

impl Url {
    pub fn empty() -> Self {
        "empty://localhost".parse().unwrap()
    }

    pub fn protocol(&self) -> &str {
        self.inner.scheme()
    }

    pub fn host(&self) -> Option<&str> {
        self.inner.host_str()
    }

    pub fn authority(&self) -> &str {
        self.inner.authority()
    }

    pub fn username(&self) -> &str {
        self.inner.username()
    }

    pub fn password(&self) -> Option<&str> {
        self.inner.password()
    }

    pub fn port(&self) -> Option<u16> {
        self.inner.port_or_known_default()
    }

    pub fn path(&self) -> &str {
        self.inner.path()
    }

    pub fn query<T: UrlParam>(&self) -> Option<T> {
        self.inner
            .query_pairs()
            .find(|(k, _)| k == T::name())
            .map(|(_, v)| T::from_str(&v).ok())
            .flatten()
    }

    pub fn query_param_by_key(&self, key: &str) -> Option<String> {
        self.inner
            .query_pairs()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.into_owned())
    }

    pub fn all_query_params(&self) -> HashMap<String, String> {
        self.inner
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect()
    }

    pub fn set_protocol(&mut self, protocol: &str) {
        let _ = self.inner.set_scheme(protocol);
    }

    pub fn set_host(&mut self, host: &str) {
        let _ = self.inner.set_host(Some(host));
    }

    pub fn set_port(&mut self, port: u16) {
        let _ = self.inner.set_port(Some(port));
    }

    pub fn set_username(&mut self, username: &str) {
        let _ = self.inner.set_username(username);
    }

    pub fn set_password(&mut self, password: &str) {
        let _ = self.inner.set_password(Some(password));
    }

    pub fn set_path(&mut self, path: &str) {
        let _ = self.inner.set_path(path);
    }

    pub fn extend_pairs(&mut self, pairs: impl Iterator<Item = (String, String)>) {
        let mut query_pairs = self.inner.query_pairs_mut();
        query_pairs.extend_pairs(pairs);
    }

    pub fn add_query_param<T: UrlParam>(&mut self, param: T) {
        let mut pairs = self.inner.query_pairs_mut();
        pairs.append_pair(T::name(), &param.as_str());
    }

    pub fn remove_query_param<T: UrlParam>(&mut self) {
        let query = self.inner.query_pairs().filter(|(k, _v)| k.ne(T::name()));
        let mut inner_url = self.inner.clone();
        inner_url.query_pairs_mut().clear().extend_pairs(query);
        self.inner = inner_url;
    }

    pub fn remove_all_param(&mut self) {
        self.inner.query_pairs_mut().clear();
    }

    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    pub fn short_url_without_query(&self) -> String {
        let mut url = self.inner.clone();
        url.set_query(Some(""));
        url.into()
    }
}

impl FromStr for Url {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Url {
            inner: url::Url::parse(s)?,
        })
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}

impl Debug for Url {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.inner, f)
    }
}

impl From<Url> for String {
    fn from(url: Url) -> Self {
        url.inner.into()
    }
}

pub trait UrlParam: FromStr {
    type TargetType;

    fn name() -> &'static str;

    fn value(&self) -> Self::TargetType;

    fn as_str(&self) -> Cow<str>;
}
