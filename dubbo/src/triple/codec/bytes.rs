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

use bytes::{Buf, BufMut, Bytes};

use super::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};

/// A codec that treats each Triple message as an opaque protobuf payload.
///
/// This is the boundary needed by higher-level runtimes such as JavaScript:
/// JS can own protobuf encoding/decoding while Rust owns Triple transport,
/// registry, routing, and connection state.
#[derive(Debug, Clone, Default)]
pub struct BytesCodec;

impl Codec for BytesCodec {
    type Encode = Bytes;
    type Decode = Bytes;

    type Encoder = BytesEncoder;
    type Decoder = BytesDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        BytesEncoder
    }

    fn decoder(&mut self) -> Self::Decoder {
        BytesDecoder
    }
}

#[derive(Debug, Clone, Default)]
pub struct BytesEncoder;

impl Encoder for BytesEncoder {
    type Item = Bytes;
    type Error = crate::status::Status;

    fn encode(&mut self, item: Self::Item, buf: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        buf.reserve(item.len());
        buf.put_slice(&item);
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct BytesDecoder;

impl Decoder for BytesDecoder {
    type Item = Bytes;
    type Error = crate::status::Status;

    fn decode(&mut self, buf: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let len = buf.remaining();
        Ok(Some(buf.copy_to_bytes(len)))
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn bytes_codec_preserves_payload() {
        let payload = Bytes::from_static(b"\x08\x96\x01");
        let mut encoded = BytesMut::new();
        let mut encoder = BytesEncoder;

        encoder
            .encode(payload.clone(), &mut EncodeBuf::new(&mut encoded))
            .unwrap();

        let len = encoded.len();
        let mut decoder = BytesDecoder;
        let decoded = decoder
            .decode(&mut DecodeBuf::new(&mut encoded, len))
            .unwrap()
            .unwrap();

        assert_eq!(decoded, payload);
    }
}
