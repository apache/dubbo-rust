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
use lazy_static::lazy_static;
use socket2::{Domain, Protocol, Socket, Type};

#[derive(Debug)]
pub struct IpStackCapability {
    pub ipv4: bool,
    pub ipv6: bool,
    pub ipv4_mapped_ipv6: bool,
}

impl IpStackCapability {
    fn probe() -> Self {
        IpStackCapability {
            ipv4: Self::probe_ipv4(),
            ipv6: Self::probe_ipv6(),
            ipv4_mapped_ipv6: Self::probe_ipv4_mapped_ipv6(),
        }
    }

    fn probe_ipv4() -> bool {
        let s = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP));
        s.is_ok()
    }

    fn probe_ipv6() -> bool {
        let s = Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP));
        let s = match s {
            Ok(s) => s,
            Err(_) => return false,
        };
        // this error is ignored in go, follow their strategy
        let _ = s.set_only_v6(true);
        let addr: std::net::SocketAddr = ([0, 0, 0, 0, 0, 0, 0, 1], 0).into();
        s.bind(&addr.into()).is_ok()
    }

    fn probe_ipv4_mapped_ipv6() -> bool {
        let s = Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP));
        let s = match s {
            Ok(s) => s,
            Err(_) => return false,
        };
        !s.only_v6().unwrap_or(true)
    }
}

pub fn probe() -> &'static IpStackCapability {
    lazy_static! {
        static ref CAPABILITY: IpStackCapability = IpStackCapability::probe();
    }
    &CAPABILITY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn tryout_probe() {
        println!("{:?}", probe());
    }
}
