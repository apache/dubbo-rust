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
use std::{collections::HashMap, fs, path::PathBuf, sync::Mutex};

use anyhow::Error;
use once_cell::sync::Lazy;
use serde_yaml::{from_slice, Value};

static YAML_VALUE_CACHE_MAP: Lazy<Mutex<HashMap<PathBuf, Value>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// parse yaml file to structs
pub fn yaml_file_parser<T>(path: PathBuf) -> Result<T, Error>
where
    T: serde::de::DeserializeOwned + std::fmt::Debug,
{
    if !path.is_file() {
        return Err(anyhow::anyhow!("path is not a file: {:?}", path));
    }
    let data = fs::read(path.as_path())?;
    Ok(from_slice(&data).unwrap())
}

// read value by a key like: logging.level
pub fn yaml_key_reader(path: PathBuf, key: &str) -> Result<Option<String>, Error> {
    if !path.is_file() {
        return Err(anyhow::anyhow!("path is not a file: {:?}", path));
    }
    let cache_map = YAML_VALUE_CACHE_MAP.lock().unwrap();
    let split_keys = key.split('.');
    let data = fs::read(path.as_path())?;
    let mut value: Value;
    match cache_map.contains_key(path.as_path()) {
        true => {
            value = cache_map.get(path.as_path()).unwrap().clone();
        }
        false => {
            value = from_slice(&data).unwrap();
        }
    }
    for key in split_keys {
        value = value[key].clone();
    }
    if value.is_null() {
        return Ok(None);
    }
    Ok(Some(value.as_str().unwrap().to_string()))
}

#[cfg(test)]
mod tests {
    use serde_yaml::Value;
    use std::collections::HashMap;

    use crate::{
        path_util::app_root_dir,
        yaml_util::{yaml_file_parser, yaml_key_reader},
    };

    #[test]
    fn test_yaml_file_parser() {
        let path = app_root_dir()
            .join("common")
            .join("utils")
            .join("tests")
            .join("application.yaml");
        let config = yaml_file_parser::<HashMap<String, Value>>(path).unwrap();
        println!("{:?}", config);
    }

    #[test]
    fn test_yaml_key_reader() {
        let path = app_root_dir()
            .join("common")
            .join("utils")
            .join("tests")
            .join("application.yaml");
        let config = yaml_key_reader(path.clone(), "logging.level").unwrap();
        println!("{:?}", config);
        let config = yaml_key_reader(path, "logging.file.path").unwrap();
        println!("{:?}", config);
    }
}
