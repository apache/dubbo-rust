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

use dubbo_base::Url;
use nacos_sdk::api::props::ClientProps;

const APP_NAME_KEY: &str = "AppName";

const UNKNOWN_APP: &str = "UnknownApp";

const NAMESPACE_KEY: &str = "namespace";

const DEFAULT_NAMESPACE: &str = "public";

const USERNAME_KEY: &str = "username";

const PASSWORD_KEY: &str = "password";

const BACKUP_KEY: &str = "backup";

const WILDCARD: &str = "*";

const RANGE_STR_SEPARATOR: &str = ",";

pub(crate) fn build_nacos_client_props(url: &Url) -> (nacos_sdk::api::props::ClientProps, bool) {
    let host = &url.ip;
    let port = &url.port;
    let backup = url
        .get_param(BACKUP_KEY)
        .map(|mut data| {
            data.insert(0, ',');
            data
        })
        .unwrap_or_default();
    let server_addr = format!("{}:{}{}", host, port, backup);

    let namespace = url
        .get_param(NAMESPACE_KEY)
        .unwrap_or_else(|| DEFAULT_NAMESPACE.to_string());
    let app_name = url
        .get_param(APP_NAME_KEY)
        .unwrap_or_else(|| UNKNOWN_APP.to_string());
    let username = url.get_param(USERNAME_KEY).unwrap_or_default();
    let password = url.get_param(PASSWORD_KEY).unwrap_or_default();

    let enable_auth = !password.is_empty() && !username.is_empty();

    // todo ext parameters

    let mut client_props = ClientProps::new();

    client_props = client_props
        .server_addr(server_addr)
        .namespace(namespace)
        .app_name(app_name)
        .auth_username(username)
        .auth_password(password);

    (client_props, enable_auth)
}

pub(crate) fn is_wildcard_str(str: &str) -> bool {
    str.eq(WILDCARD)
}

pub(crate) fn is_range_str(str: &str) -> bool {
    let ret = str.split(RANGE_STR_SEPARATOR);
    let count = ret.count();
    count > 1
}

pub(crate) fn is_concrete_str(str: &str) -> bool {
    !is_wildcard_str(str) && !is_range_str(str)
}

pub(crate) fn match_range(range: &str, value: &str) -> bool {
    if range.is_empty() {
        return true;
    }

    if !is_range_str(range) {
        return false;
    }

    range
        .split(RANGE_STR_SEPARATOR)
        .any(|data| (*data).eq(value))
}
