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
// type for registryName;can be customized;RegistryKey eg. zookeeper/nacos/consul
pub type RegistryId = String;
pub type RegistryKey = String;
// service/application
pub type RegistryType = String;
// protocolKey defined in protocol layer, mean specified protocol
pub type ServiceName = String;
pub type ServiceKey = String;
//
pub type ProtocolKey = String;
pub type GroupId = String;
pub type Version = String;

pub type InterfaceName = String;

pub type ClusterStrategy = String;

pub type ParamKey = String;