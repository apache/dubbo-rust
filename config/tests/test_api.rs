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
    use std::env;
    use std::sync::Once;

    use ctor::ctor;

    use config::get_root_config;
    use config::location::set_config_file_path;

    static INIT: Once = Once::new();

    #[ctor]
    fn setup() {
        INIT.call_once(|| {
            set_config_file_path(format!(
                "{}/{}",
                env::current_dir()
                    .unwrap()
                    .into_os_string()
                    .to_str()
                    .unwrap(),
                "tests"
            ));
        });
    }

    #[test]
    fn test_api_overwrite_yaml() {
        let root_config = get_root_config();
        println!("{:#?}", root_config);
        root_config
            .lock()
            .unwrap()
            .protocols
            .get_mut("dubbo")
            .unwrap()
            .port = 20987;
        assert_eq!(
            root_config
                .lock()
                .unwrap()
                .protocols
                .get("dubbo")
                .unwrap()
                .port,
            20987
        );
    }
}
