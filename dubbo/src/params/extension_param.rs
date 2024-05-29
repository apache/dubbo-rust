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
use crate::{url::UrlParam, StdError};
use std::{borrow::Cow, convert::Infallible, str::FromStr};

pub struct ExtensionName(String);

impl ExtensionName {
    pub fn new(name: String) -> Self {
        ExtensionName(name)
    }
}

impl UrlParam for ExtensionName {
    type TargetType = String;

    fn name() -> &'static str {
        "extension-name"
    }

    fn value(&self) -> Self::TargetType {
        self.0.clone()
    }

    fn as_str(&self) -> Cow<str> {
        self.0.as_str().into()
    }
}

impl FromStr for ExtensionName {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ExtensionName::new(s.to_string()))
    }
}

pub enum ExtensionType {
    Registry,
    Invoker,
}

impl UrlParam for ExtensionType {
    type TargetType = String;

    fn name() -> &'static str {
        "extension-type"
    }

    fn value(&self) -> Self::TargetType {
        match self {
            ExtensionType::Registry => "registry".to_owned(),
            ExtensionType::Invoker => "invoker".to_owned(),
        }
    }

    fn as_str(&self) -> Cow<str> {
        match self {
            ExtensionType::Registry => Cow::Borrowed("registry"),
            ExtensionType::Invoker => Cow::Borrowed("invoker"),
        }
    }
}

impl FromStr for ExtensionType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "registry" => Ok(ExtensionType::Registry),
            _ => panic!("the extension type enum is not in range"),
        }
    }
}
