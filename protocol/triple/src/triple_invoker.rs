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
use dubbo_base::{Node, Url};

use protocol_base::{
    invocation::BoxInvocation,
    invoker::{BaseInvoker, Invoker},
};
use std::sync::Arc;

pub struct TripleInvoker {
    base: BaseInvoker,
}

impl Invoker for TripleInvoker {
    type Output = ();

    fn invoke(&self, _invocation: BoxInvocation) -> Self::Output {
        todo!()
    }
}

impl Node for TripleInvoker {
    fn get_url(&self) -> Arc<Url> {
        self.base.get_url()
    }

    fn is_available(&self) -> bool {
        self.base.is_available()
    }

    fn destroy(&self) {
        todo!()
    }

    fn is_destroyed(&self) -> bool {
        self.base.is_destroyed()
    }
}
