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

mod tag;

use crate::invocation::Invocation;
use dubbo_base::Url;
use std::{
    cmp::Ordering,
    fmt::{Debug, Formatter},
    sync::Arc,
};

pub use tag::TagRouter;

pub trait Router {
    fn get_url(&self) -> Option<&Url>;
    fn get_priority(&self) -> i32;
    fn route(&self, invokers: &Vec<Url>, url: Url, invocation: Arc<dyn Invocation>) -> Vec<Url>;
}

/* ------------- RouterWrapper for extending ability of Router ------------- */

pub struct RouterWrapper(pub Box<dyn Router>);

impl RouterWrapper {
    pub fn compare_priority(&self, other: &Self) -> Ordering {
        match (self.0.get_priority(), other.0.get_priority()) {
            (a, b) if a > b => Ordering::Greater,
            (a, b) if a < b => Ordering::Less,
            (a, b) if a == b => Ordering::Equal,
            _ => unreachable!(),
        }
    }

    pub fn get_url(&self) -> Option<&Url> {
        self.0.get_url()
    }

    pub fn get_priority(&self) -> i32 {
        self.0.get_priority()
    }

    pub fn route(
        &self,
        invokers: &Vec<Url>,
        url: Url,
        invocation: Arc<dyn Invocation>,
    ) -> Vec<Url> {
        self.0.route(invokers, url, invocation)
    }
}

/* ---------------------- Impl Debug for RouterWrapper --------------------- */

impl Debug for RouterWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Router{{url: {:?}, priority: {:?}}}",
            self.get_url(),
            self.get_priority()
        )
    }
}

/* ----------------------- Impl Ord for RouterWrapper ---------------------- */

impl Eq for RouterWrapper {}

impl PartialEq<Self> for RouterWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.compare_priority(other) == Ordering::Equal
    }
}

impl PartialOrd<Self> for RouterWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.compare_priority(other))
    }
}

impl Ord for RouterWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare_priority(other)
    }
}
