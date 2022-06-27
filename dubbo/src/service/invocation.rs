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

use tonic::metadata::MetadataMap;

pub struct Request<T> {
    pub message: T,
    pub metadata: MetadataMap,
}

impl<T> Request<T> {
    pub fn new(message: T) -> Request<T> {
        Self {
            message,
            metadata: MetadataMap::new(),
        }
    }

    pub fn into_parts(self) -> (MetadataMap, T) {
        (self.metadata, self.message)
    }
}

pub struct Response<T> {
    message: T,
    metadata: MetadataMap,
}

impl<T> Response<T> {
    pub fn new(message: T) -> Response<T> {
        Self {
            message,
            metadata: MetadataMap::new(),
        }
    }

    pub fn from_parts(metadata: MetadataMap, message: T) -> Self {
        Self { message, metadata }
    }

    pub fn into_parts(self) -> (MetadataMap, T) {
        (self.metadata, self.message)
    }
}
