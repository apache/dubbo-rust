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
use std::fs;
use std::path::PathBuf;

use anyhow::Error;
use serde_yaml::from_slice;

use logger::tracing;

// parse yaml file to structs
pub fn yaml_file_parser<T>(path: PathBuf) -> Result<T, Error>
where
    T: serde::de::DeserializeOwned + std::fmt::Debug,
{
    if !path.is_file() {
        return Err(anyhow::anyhow!("path is not a file: {:?}", path));
    }
    let data = fs::read(path.as_path())?;
    tracing::debug!("config data: {:?}", String::from_utf8(data.clone()));
    Ok(from_slice(&data).unwrap())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::path_util::app_root_dir;
    use crate::yaml_util::yaml_file_parser;

    #[test]
    fn test_yaml_file_parser() {
        let path = app_root_dir()
            .join("common")
            .join("utils")
            .join("tests")
            .join("application.yaml");
        let config = yaml_file_parser::<HashMap<String, HashMap<String, String>>>(path).unwrap();
        println!("{:?}", config);
    }
}
