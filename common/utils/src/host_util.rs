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
use std::net::IpAddr;

use port_selector::is_free;

pub use port_selector::Port;

// get local ip for linux/macos/windows
#[allow(dead_code)]
pub(crate) fn local_ip() -> IpAddr {
    local_ip_address::local_ip().unwrap()
}

#[allow(dead_code)]
pub(crate) fn is_free_port(port: Port) -> bool {
    is_free(port)
}

// scan from the give port
#[allow(dead_code)]
pub(crate) fn scan_free_port(port: Port) -> Port {
    for selected_port in port..65535 {
        if is_free_port(selected_port) {
            return selected_port;
        } else {
            continue;
        }
    }
    port
}

#[cfg(test)]
mod tests {
    use local_ip_address::list_afinet_netifas;

    use super::*;

    #[test]
    fn test_local_ip() {
        let ip = local_ip();
        println!("ip: {}", ip);
    }

    #[test]
    fn test_local_addresses() {
        let network_interfaces = list_afinet_netifas().unwrap();
        for (name, ip) in network_interfaces.iter() {
            println!("{}:\t{:?}", name, ip);
        }
    }

    #[test]
    fn test_scan_free_port() {
        let free_port = scan_free_port(7890);
        println!("{}", free_port);
    }
}
