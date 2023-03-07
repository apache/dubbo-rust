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
use std::env;

pub fn get_env_value(env_key: &str) -> Option<String> {
    env::var(env_key).ok()
}

pub fn get_env_value_for_i32(env_key: &str) -> Option<i32> {
    get_env_value(env_key).map(|v| v.parse::<i32>().unwrap())
}

#[cfg(test)]
mod tests {
    use crate::env_util::{get_env_value, get_env_value_for_i32};
    use std::env;

    #[test]
    fn test_get_env_value() {
        env::set_var("TEST_ENV", "testxxx1");
        env::set_var("TEST_ENV3", "999");
        assert!(get_env_value("TEST_ENV").is_some());
        assert!(get_env_value("TEST_ENV2").is_none());
        assert_eq!(get_env_value_for_i32("TEST_ENV3"), Some(999_i32))
    }
}
