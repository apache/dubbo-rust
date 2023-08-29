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

use rustls_pemfile::{certs, ec_private_keys, pkcs8_private_keys, rsa_private_keys};
use std::{
    fs::File,
    io::{self, BufReader, Cursor, Read},
    path::Path,
};
use tokio_rustls::rustls::{Certificate, PrivateKey};

pub fn load_certs(path: &Path) -> io::Result<Vec<Certificate>> {
    certs(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
        .map(|mut certs| certs.drain(..).map(Certificate).collect())
}

pub fn load_keys(path: &Path) -> io::Result<Vec<PrivateKey>> {
    let file = &mut BufReader::new(File::open(path)?);
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let mut cursor = Cursor::new(data);

    let parsers = [rsa_private_keys, pkcs8_private_keys, ec_private_keys];

    for parser in &parsers {
        if let Ok(mut key) = parser(&mut cursor) {
            if !key.is_empty() {
                return Ok(key.drain(..).map(PrivateKey).collect());
            }
        }
        cursor.set_position(0);
    }

    Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
}
