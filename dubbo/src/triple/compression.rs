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

use std::collections::HashMap;

use bytes::{Buf, BufMut, BytesMut};
use flate2::{
    read::{GzDecoder, GzEncoder},
    Compression,
};
use lazy_static::lazy_static;

pub const GRPC_ACCEPT_ENCODING: &str = "grpc-accept-encoding";
pub const GRPC_ENCODING: &str = "grpc-encoding";

#[derive(Debug, Clone, Copy)]
pub enum CompressionEncoding {
    Gzip,
}

lazy_static! {
    pub static ref COMPRESSIONS: HashMap<String, Option<CompressionEncoding>> = {
        let mut v = HashMap::new();
        v.insert("gzip".to_string(), Some(CompressionEncoding::Gzip));
        v
    };
}

impl CompressionEncoding {
    pub fn from_accept_encoding(header: &http::HeaderMap) -> Option<CompressionEncoding> {
        let accept_encoding = header.get(GRPC_ACCEPT_ENCODING)?;
        let encodings = accept_encoding.to_str().ok()?;

        encodings
            .trim()
            .split(',')
            .map(|s| s.trim())
            .into_iter()
            .find_map(|s| match s {
                "gzip" => Some(CompressionEncoding::Gzip),
                _ => None,
            })
    }

    pub fn into_header_value(self) -> http::HeaderValue {
        match self {
            CompressionEncoding::Gzip => http::HeaderValue::from_static("gzip"),
        }
    }
}

pub fn compress(
    encoding: CompressionEncoding,
    src: &mut BytesMut,
    dst: &mut BytesMut,
    len: usize,
) -> Result<(), std::io::Error> {
    dst.reserve(len);

    match encoding {
        CompressionEncoding::Gzip => {
            let mut en = GzEncoder::new(src.reader(), Compression::default());

            let mut dst_writer = dst.writer();

            std::io::copy(&mut en, &mut dst_writer)?;
        }
    }

    Ok(())
}

pub fn decompress(
    encoding: CompressionEncoding,
    src: &mut BytesMut,
    dst: &mut BytesMut,
    len: usize,
) -> Result<(), std::io::Error> {
    let capacity = len * 2;
    dst.reserve(capacity);

    match encoding {
        CompressionEncoding::Gzip => {
            let mut de = GzDecoder::new(src.reader());

            let mut dst_writer = dst.writer();

            std::io::copy(&mut de, &mut dst_writer)?;
        }
    }
    Ok(())
}

#[test]
fn test_compress() {
    let mut src = BytesMut::with_capacity(super::consts::BUFFER_SIZE);
    src.put(&b"test compress"[..]);
    let mut dst = BytesMut::new();
    let len = src.len();
    src.reserve(len);

    compress(CompressionEncoding::Gzip, &mut src, &mut dst, len).unwrap();
    println!("src: {:?}, dst: {:?}", src, dst);

    let mut de_dst = BytesMut::with_capacity(super::consts::BUFFER_SIZE);
    let de_len = dst.len();
    decompress(CompressionEncoding::Gzip, &mut dst, &mut de_dst, de_len).unwrap();

    println!("src: {:?}, dst: {:?}", dst, de_dst);
}
