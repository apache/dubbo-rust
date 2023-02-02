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

use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use http::Uri;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Url {
    pub raw_url_string: String,
    // value of scheme is different to protocol name, eg. triple -> tri://
    pub scheme: String,
    pub location: String,
    pub ip: String,
    pub port: String,
    // serviceKey format in dubbo java and go '{group}/{interfaceName}:{version}'
    pub service_key: String,
    pub params: HashMap<String, String>,
}

impl Url {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_url(url: &str) -> Option<Self> {
        // url: tri://127.0.0.1:8888/helloworld.Greeter
        let uri = url
            .parse::<http::Uri>()
            .map_err(|err| {
                tracing::error!("fail to parse url({}), err: {:?}", url, err);
            })
            .unwrap();
        let query = uri.path_and_query().unwrap().query();
        Some(Self {
            raw_url_string: url.to_string(),
            scheme: uri.scheme_str()?.to_string(),
            ip: uri.authority()?.host().to_string(),
            port: uri.authority()?.port()?.to_string(),
            location: uri.authority()?.to_string(),
            service_key: uri.path().trim_start_matches('/').to_string(),
            params: if let Some(..) = query {
                Url::decode(query.unwrap())
            } else {
                HashMap::new()
            },
        })
    }

    pub fn get_service_name(&self) -> String {
        self.service_key.clone()
    }

    pub fn get_param(&self, key: &str) -> Option<String> {
        self.params.get(key).cloned()
    }

    fn encode_param(&self) -> String {
        let mut params_vec: Vec<String> = Vec::new();
        for (k, v) in self.params.iter() {
            // let tmp = format!("{}={}", k, v);
            params_vec.push(format!("{}={}", k, v));
        }
        params_vec.join("&")
    }

    pub fn params_count(&self) -> usize {
        self.params.len()
    }

    fn decode(raw_query_string: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let p: Vec<String> = raw_query_string
            .split('&')
            .map(|v| v.trim().to_string())
            .collect();
        for v in p.iter() {
            let values: Vec<String> = v.split('=').map(|v| v.trim().to_string()).collect();
            if values.len() != 2 {
                continue;
            }
            params.insert(values[0].clone(), values[1].clone());
        }
        params
    }

    pub fn set_param(&mut self, key: &str, value: &str) {
        self.params.insert(key.to_string(), value.to_string());
        self.renew_raw_url_string();
    }

    pub fn raw_url_string(&self) -> String {
        self.raw_url_string.clone()
    }

    fn renew_raw_url_string(&mut self) {
        self.raw_url_string = format!(
            "{}://{}:{}={}",
            self.scheme,
            self.ip,
            self.port,
            self.encode_param()
        )
    }

    // short_url is used for tcp listening
    pub fn short_url(&self) -> String {
        format!("{}://{}:{}", self.scheme, self.ip, self.port)
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.raw_url_string().as_str())
    }
}

impl Into<Uri> for Url {
    fn into(self) -> Uri {
        self.raw_url_string.parse::<Uri>().unwrap()
    }
}

impl From<&str> for Url {
    fn from(url: &str) -> Self {
        Url::from_url(url).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_url() {
        let mut u1 = Url::from_url("tri://127.0.0.1:20000/com.ikurento.user.UserProvider?anyhost=true&\
        application=BDTService&category=providers&default.timeout=10000&dubbo=dubbo-provider-golang-1.0.0&\
        environment=dev&interface=com.ikurento.user.UserProvider&ip=192.168.56.1&methods=GetUser%2C&\
        module=dubbogo+user-info+server&org=ikurento.com&owner=ZX&pid=1447&revision=0.0.1&\
        side=provider&timeout=3000&timestamp=1556509797245");
        assert_eq!(
            u1.as_ref().unwrap().service_key,
            "com.ikurento.user.UserProvider"
        );
        assert_eq!(
            u1.as_ref().unwrap().get_param("anyhost").unwrap().as_str(),
            "true"
        );
        assert_eq!(
            u1.as_ref()
                .unwrap()
                .get_param("default.timeout")
                .unwrap()
                .as_str(),
            "10000"
        );
        assert_eq!(u1.as_ref().unwrap().scheme, "tri");
        assert_eq!(u1.as_ref().unwrap().ip, "127.0.0.1");
        assert_eq!(u1.as_ref().unwrap().port, "20000");
        assert_eq!(u1.as_ref().unwrap().params_count(), 17);
        u1.as_mut().unwrap().set_param("key1", "value1");
        assert_eq!(
            u1.as_ref().unwrap().get_param("key1").unwrap().as_str(),
            "value1"
        );
    }
}
