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

//type ServiceDesc struct {
//     ServiceName string
//     // The pointer to the service interface. Used to check whether the user
//     // provided implementation satisfies the interface requirements.
//     HandlerType interface{}
//     Methods     []MethodDesc
//     Streams     []StreamDesc
//     Metadata    interface{}
// }

use std::collections::HashMap;

pub struct ServiceDesc {
    service_name: String,
    // methods: HashMap<String, String> // "/Greeter/hello": "unary"
}

impl ServiceDesc {
    pub fn new(service_name: String, _methods: HashMap<String, String>) -> Self {
        Self { service_name }
    }

    pub fn get_service_name(&self) -> String {
        self.service_name.clone()
    }
}
