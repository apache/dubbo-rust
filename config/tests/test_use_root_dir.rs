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

#[cfg(test)]
mod tests_api {
    use anyhow::Error;

    use config::api::ConfigApi;
    use config::get_root_config;
    use config::types::protocol::Protocol;
    use config::types::registry::Registry;

    #[test]
    fn test_dubbo_protocol_set_overwrite_yaml_by_api() -> Result<(), Error> {
        let config_wrapper = get_root_config();
        let old_config: Protocol = config_wrapper.dubbo_protocol_get("dubbo")?;
        assert_eq!(old_config.port, "8888");
        println!("{}", old_config.ip);
        config_wrapper.dubbo_protocol_set("dubbo", "ip", "122.22.22.22")?;
        config_wrapper.dubbo_protocol_set("dubbo", "port", "111")?;
        config_wrapper.dubbo_protocol_set("dubbo", "name", "dubbo")?;
        config_wrapper.dubbo_protocol_set("dubbo", "nam1e", "dubbo")?;
        let new_config: Protocol = config_wrapper.dubbo_protocol_get("dubbo")?;
        assert_eq!(new_config.port, "111".to_string());
        assert_eq!(new_config.name, "dubbo".to_string());
        assert_eq!(
            new_config.params.get("nam1e").unwrap().clone(),
            "dubbo".to_string()
        );
        Ok(())
    }

    #[test]
    fn test_registry_config() -> Result<(), Error> {
        let zk_config: Registry = get_root_config().dubbo_registry_get("demoZK")?;
        assert_eq!("zookeeper", zk_config.protocol);
        Ok(())
    }

    #[test]
    fn test_default_value() -> Result<(), Error> {
        let zk_config: Registry = get_root_config().dubbo_registry_get("demoZK")?;
        assert_eq!("3000", zk_config.timeout);
        Ok(())
    }
}
