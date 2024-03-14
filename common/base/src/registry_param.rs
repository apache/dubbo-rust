use crate::{url::UrlParam, StdError, Url};
use std::{borrow::Cow, convert::Infallible, str::FromStr};

pub struct RegistryUrl(Url);

impl RegistryUrl {
    pub fn new(url: Url) -> Self {
        Self(url)
    }
}

impl UrlParam for RegistryUrl {
    type TargetType = Url;

    fn name() -> &'static str {
        "registry"
    }

    fn value(&self) -> Self::TargetType {
        self.0.clone()
    }

    fn as_str(&self) -> Cow<str> {
        self.0.as_str().into()
    }
}

impl FromStr for RegistryUrl {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

pub struct ServiceNamespace(String);

impl ServiceNamespace {
    pub fn new(namespace: String) -> Self {
        Self(namespace)
    }
}

impl UrlParam for ServiceNamespace {
    type TargetType = String;

    fn name() -> &'static str {
        "namespace"
    }

    fn value(&self) -> Self::TargetType {
        self.0.clone()
    }

    fn as_str(&self) -> Cow<str> {
        self.0.as_str().into()
    }
}

impl FromStr for ServiceNamespace {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Default for ServiceNamespace {
    fn default() -> Self {
        Self("public".to_string())
    }
}

pub struct AppName(String);

impl AppName {
    pub fn new(app_name: String) -> Self {
        Self(app_name)
    }
}

impl UrlParam for AppName {
    type TargetType = String;

    fn name() -> &'static str {
        "app_name"
    }

    fn value(&self) -> Self::TargetType {
        self.0.clone()
    }

    fn as_str(&self) -> Cow<str> {
        self.0.as_str().into()
    }
}

impl FromStr for AppName {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Default for AppName {
    fn default() -> Self {
        Self("UnknownApp".to_string())
    }
}

pub struct InterfaceName(String);

impl InterfaceName {
    pub fn new(interface_name: String) -> Self {
        Self(interface_name)
    }
}

impl UrlParam for InterfaceName {
    type TargetType = String;

    fn name() -> &'static str {
        "interface"
    }

    fn value(&self) -> Self::TargetType {
        self.0.clone()
    }

    fn as_str(&self) -> Cow<str> {
        self.0.as_str().into()
    }
}

impl FromStr for InterfaceName {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Default for InterfaceName {
    fn default() -> Self {
        Self("".to_string())
    }
}

pub struct Category(String);

impl Category {
    pub fn new(category: String) -> Self {
        Self(category)
    }
}

impl UrlParam for Category {
    type TargetType = String;

    fn name() -> &'static str {
        "category"
    }

    fn value(&self) -> Self::TargetType {
        self.0.clone()
    }

    fn as_str(&self) -> Cow<str> {
        self.0.as_str().into()
    }
}

impl FromStr for Category {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Default for Category {
    fn default() -> Self {
        Self("".to_string())
    }
}

pub struct Version(String);

impl Version {
    pub fn new(version: String) -> Self {
        Self(version)
    }
}

impl UrlParam for Version {
    type TargetType = String;

    fn name() -> &'static str {
        "version"
    }

    fn value(&self) -> Self::TargetType {
        self.0.clone()
    }

    fn as_str(&self) -> Cow<str> {
        self.0.as_str().into()
    }
}

impl FromStr for Version {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Default for Version {
    fn default() -> Self {
        Self("".to_string())
    }
}

pub struct Group(String);

impl Group {
    pub fn new(group: String) -> Self {
        Self(group)
    }
}

impl UrlParam for Group {
    type TargetType = String;

    fn name() -> &'static str {
        "group"
    }

    fn value(&self) -> Self::TargetType {
        self.0.clone()
    }

    fn as_str(&self) -> Cow<str> {
        self.0.as_str().into()
    }
}

impl FromStr for Group {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Default for Group {
    fn default() -> Self {
        Self("".to_string())
    }
}

pub enum Side {
    Provider,
    Consumer,
}

impl UrlParam for Side {
    type TargetType = String;

    fn name() -> &'static str {
        "side"
    }

    fn value(&self) -> Self::TargetType {
        match self {
            Side::Consumer => "consumer".to_owned(),
            Side::Provider => "provider".to_owned(),
        }
    }

    fn as_str(&self) -> Cow<str> {
        match self {
            Side::Consumer => Cow::Borrowed("consumer"),
            Side::Provider => Cow::Borrowed("provider"),
        }
    }
}

impl FromStr for Side {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "consumer" => Ok(Side::Consumer),
            "provider" => Ok(Side::Provider),
            _ => Ok(Side::Consumer),
        }
    }
}

impl Default for Side {
    fn default() -> Self {
        Side::Consumer
    }
}

pub struct StaticInvokerUrls(String);

impl UrlParam for StaticInvokerUrls {
    type TargetType = Vec<Url>;

    fn name() -> &'static str {
        "static-invoker-urls"
    }

    fn value(&self) -> Self::TargetType {
        self.0.split(",").map(|url| url.parse().unwrap()).collect()
    }

    fn as_str(&self) -> Cow<str> {
        Cow::Borrowed(&self.0)
    }
}

impl FromStr for StaticInvokerUrls {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Default for StaticInvokerUrls {
    fn default() -> Self {
        Self(String::default())
    }
}
