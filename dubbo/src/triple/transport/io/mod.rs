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

use std::pin::Pin;

use hyper::client::connect::Connection;
use tokio::io::{AsyncRead, AsyncWrite};

pub struct BoxIO {
    reader: Box<dyn AsyncRead + Unpin + Send + 'static>,
    writer: Box<dyn AsyncWrite + Unpin + Send + 'static>,
}

impl BoxIO {
    pub fn new(io: impl AsyncWrite + AsyncRead + Unpin + Send + 'static) -> Self {
        let (r, w) = tokio::io::split(io);
        BoxIO {
            reader: Box::new(r),
            writer: Box::new(w),
        }
    }
}

impl AsyncWrite for BoxIO {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let s = &mut *self;
        Pin::new(&mut s.writer).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let s = &mut *self;
        Pin::new(&mut s.writer).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let s = &mut *self;
        Pin::new(&mut s.writer).poll_shutdown(cx)
    }
}

impl AsyncRead for BoxIO {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let s = &mut *self;
        Pin::new(&mut s.reader).poll_read(cx, buf)
    }
}

/// for connector
impl Connection for BoxIO {
    fn connected(&self) -> hyper::client::connect::Connected {
        hyper::client::connect::Connected::new()
    }
}
