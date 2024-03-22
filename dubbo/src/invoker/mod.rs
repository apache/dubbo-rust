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
use crate::{codegen::TripleInvoker, invoker::clone_invoker::CloneInvoker, svc::NewService};

pub mod clone_body;
pub mod clone_invoker;

pub struct NewInvoker;

impl NewService<String> for NewInvoker {
    type Service = CloneInvoker<TripleInvoker>;

    fn new_service(&self, url: String) -> Self::Service {
        // todo create another invoker by url protocol

        let url = url.parse().unwrap();
        CloneInvoker::new(TripleInvoker::new(url))
    }
}
