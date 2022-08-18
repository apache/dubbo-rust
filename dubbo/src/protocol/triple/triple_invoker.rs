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

use crate::common::url::Url;
use crate::protocol::Invoker;

#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct TripleInvoker {
    url: Url,
}

impl TripleInvoker {
    pub fn new(url: Url) -> TripleInvoker {
        Self { url }
    }
}

impl Invoker for TripleInvoker {
    fn invoke<M1>(
        &self,
        _req: crate::protocol::invocation::Request<M1>,
    ) -> crate::protocol::invocation::Response<String>
    where
        M1: Send + 'static,
    {
        todo!()
    }

    fn is_available(&self) -> bool {
        todo!()
    }

    fn destroy(&self) {
        todo!()
    }

    fn get_url(&self) -> Url {
        todo!()
    }
}
