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



//
// pub type Registries = Arc<Mutex<HashMap<String, ArcRegistry>>>;
//
// pub const DEFAULT_REGISTRY_KEY: &str = "default";
//
// pub trait RegistriesOperation {
//     fn get(&self, registry_key: &str) -> ArcRegistry;
//     fn insert(&self, registry_key: String, registry: ArcRegistry);
//     fn default_registry(&self) -> ArcRegistry;
// }
//
// impl RegistriesOperation for Registries {
//     fn get(&self, registry_key: &str) -> ArcRegistry {
//         self.as_ref()
//             .lock()
//             .unwrap()
//             .get(registry_key)
//             .unwrap()
//             .clone()
//     }
//
//     fn insert(&self, registry_key: String, registry: ArcRegistry) {
//         self.as_ref().lock().unwrap().insert(registry_key, registry);
//     }
//
//     fn default_registry(&self) -> ArcRegistry {
//         let guard = self.as_ref().lock().unwrap();
//         let (_, result) = guard
//             .iter()
//             .find_or_first(|e| e.0 == DEFAULT_REGISTRY_KEY)
//             .unwrap()
//             .to_owned();
//         result.clone()
//     }
// }
