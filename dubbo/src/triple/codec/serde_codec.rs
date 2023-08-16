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

use std::marker::PhantomData;

use bytes::{Buf, BufMut};
use serde::{Deserialize, Serialize};

use super::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};

#[derive(Debug)]
pub struct SerdeCodec<T, U> {
    _pd: PhantomData<(T, U)>,
}

impl<T, U> Default for SerdeCodec<T, U> {
    fn default() -> Self {
        Self { _pd: PhantomData }
    }
}

impl<'a, T, U> Codec for SerdeCodec<T, U>
where
    T: Serialize + Send + 'static,
    U: Deserialize<'a> + Send + 'static,
{
    type Encode = T;

    type Decode = U;

    type Encoder = SerdeEncoder<T>;

    type Decoder = SerdeDecoder<U>;

    fn encoder(&mut self) -> Self::Encoder {
        SerdeEncoder(PhantomData)
    }

    fn decoder(&mut self) -> Self::Decoder {
        SerdeDecoder(PhantomData)
    }
}

#[derive(Debug, Clone)]
pub struct SerdeEncoder<T>(PhantomData<T>);

impl<T: Serialize> Encoder for SerdeEncoder<T> {
    type Item = T;

    type Error = crate::status::Status;

    fn encode(&mut self, item: Self::Item, dst: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        item.serialize(&mut serde_json::Serializer::new(dst.writer()))
            .expect("failed to searialize");

        Ok(())
    }
}

pub struct SerdeDecoder<U>(PhantomData<U>);

impl<'a, U: Deserialize<'a>> Decoder for SerdeDecoder<U> {
    type Item = U;

    type Error = crate::status::Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let value = src.chunk().to_owned();
        let mut msg = vec![0u8; value.len()];
        src.copy_to_slice(&mut msg);

        let mut de = serde_json::Deserializer::from_reader(msg.reader());
        Ok(Some(U::deserialize(&mut de).unwrap()))
    }
}
