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

use std::sync::Arc;

use anyhow::{anyhow, Error};
use bytes::{self, Bytes};
use dashmap::DashMap;
use protocol_base::ProtocolName;

use crate::{error::CodecError, Response};

#[derive(Clone)]
pub struct BoxedCodec(Arc<dyn Codec>);

impl BoxedCodec {
    pub fn new(codec: Arc<dyn Codec>) -> Self {
        BoxedCodec(codec)
    }
}

pub trait Codec: Sync + Send {
    fn encode_request(&self) -> Result<Bytes, CodecError>;
    fn encode_response(&self) -> Result<Bytes, CodecError>;
    fn decode(&self, bytes: Bytes) -> Result<CodecResult, CodecError>;
}

pub struct CodecRegistry {
    registry: DashMap<ProtocolName, BoxedCodec>,
}

#[derive(Default)]
pub struct CodecResult {
    is_request: bool, // heartbeat flag
    result: Option<Response>,
}

impl Default for CodecRegistry {
    fn default() -> Self {
        CodecRegistry {
            registry: DashMap::new(),
        }
    }
}
impl CodecRegistry {
    pub fn get_codec(&self, protocol: ProtocolName) -> Option<BoxedCodec> {
        let registry_map = &self.registry;
        if let true = registry_map.contains_key(protocol) {
            let option = registry_map.get(protocol);
            let codec = option.as_deref().unwrap();
            Some(codec.clone())
        } else {
            None
        }
    }
    pub fn set_codec(
        &mut self,
        protocol: ProtocolName,
        codec: BoxedCodec,
    ) -> anyhow::Result<(), CodecError> {
        if let true = self.registry.contains_key(protocol) {
            return Err(CodecError::RegistryExistsProtocol(protocol));
        } else {
            self.registry.insert(protocol, codec);
        }
        Ok(())
    }

    pub fn is_registered(&self, protocol: ProtocolName) -> bool {
        self.registry.contains_key(protocol)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bytes::Bytes;

    use crate::{
        codec::{BoxedCodec, CodecRegistry, CodecResult},
        error::CodecError,
        Codec,
    };

    #[derive(Default)]
    struct TestCodec;
    impl Codec for TestCodec {
        fn encode_request(&self) -> Result<Bytes, CodecError> {
            Ok(Bytes::new())
        }

        fn encode_response(&self) -> Result<Bytes, CodecError> {
            Ok(Bytes::new())
        }

        fn decode(&self, bytes: Bytes) -> Result<CodecResult, CodecError> {
            Ok(CodecResult::default())
        }
    }

    #[test]
    fn test_registry() {
        let mut codec_registry = CodecRegistry::default();
        codec_registry
            .set_codec("test", BoxedCodec(Arc::new(TestCodec::default())))
            .unwrap();
        assert!(codec_registry.is_registered("test"));
    }
}
