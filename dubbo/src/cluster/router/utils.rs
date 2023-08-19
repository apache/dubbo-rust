use dubbo_base::Url;
use std::{collections::HashMap, string::String};

pub fn to_original_map(url: Url) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();
    result.insert("scheme".parse().unwrap(), url.scheme);
    result.insert("location".parse().unwrap(), url.location);
    result.insert("ip".parse().unwrap(), url.ip);
    result.insert("port".parse().unwrap(), url.port);
    result.insert("service_name".parse().unwrap(), url.service_name);
    result.insert("service_key".parse().unwrap(), url.service_key);
    for (key, value) in url.params {
        result.insert(key, value);
    }
    result
}
