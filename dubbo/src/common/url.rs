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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Url {
    pub uri: String,
    pub protocol: String,
    pub location: String,
    pub ip: String,
    pub port: String,
    pub service_key: Vec<String>,
    pub params: HashMap<String, String>,
}

impl Url {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_url(url: &str) -> Option<Self> {
        // url: triple://127.0.0.1:8888/helloworld.Greeter
        let uri = url
            .parse::<http::Uri>()
            .map_err(|err| {
                tracing::error!("fail to parse url({}), err: {:?}", url, err);
            })
            .unwrap();
        Some(Self {
            uri: uri.to_string(),
            protocol: uri.scheme_str()?.to_string(),
            ip: uri.authority()?.host().to_string(),
            port: uri.authority()?.port()?.to_string(),
            location: uri.authority()?.to_string(),
            service_key: uri
                .path()
                .trim_start_matches('/')
                .split(',')
                .map(|x| x.to_string())
                .collect::<Vec<_>>(),
            params: HashMap::new(),
        })
    }

    pub fn get_service_name(&self) -> Vec<String> {
        self.service_key.clone()
    }

    pub fn get_param(&self, key: String) -> Option<String> {
        self.params.get(&key).cloned()
    }

    pub fn encode_param(&self) -> String {
        let mut params_vec: Vec<String> = Vec::new();
        for (k, v) in self.params.iter() {
            // let tmp = format!("{}={}", k, v);
            params_vec.push(format!("{}={}", k, v));
        }
        params_vec.join("&")
    }

    pub fn decode(&mut self, params: String) {
        let p: Vec<String> = params.split('&').map(|v| v.trim().to_string()).collect();
        for v in p.iter() {
            let values: Vec<String> = v.split('=').map(|v| v.trim().to_string()).collect();
            if values.len() != 2 {
                continue;
            }
            self.params.insert(values[0].clone(), values[1].clone());
        }
    }

    pub fn to_url(&self) -> String {
        format!("{}://{}:{}", self.protocol, self.ip, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_url() {
        let u1 = Url::from_url("triple://127.0.0.1:8888/helloworld.Greeter");
        println!("{:?}", u1.unwrap().get_service_name())
    }

    #[test]
    fn test_encode_params() {
        let mut u = Url::default();
        u.params.insert("method".to_string(), "GET".to_string());
        u.params.insert("args".to_string(), "GET".to_string());

        let en = u.encode_param();
        println!("encode_params: {:?}", en);

        let mut u1 = Url::default();
        u1.decode(en);
        println!("decode_params: {:?}", u1);
        assert_eq!(u1, u);
    }
}
