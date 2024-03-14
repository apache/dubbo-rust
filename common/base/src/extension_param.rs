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
}

impl UrlParam for ExtensionType {
    type TargetType = String;

    fn name() -> &'static str {
        "extension-type"
    }

    fn value(&self) -> Self::TargetType {
        match self {
            ExtensionType::Registry => "registry".to_owned(),
        }
    }

    fn as_str(&self) -> Cow<str> {
        match self {
            ExtensionType::Registry => Cow::Borrowed("registry"),
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
