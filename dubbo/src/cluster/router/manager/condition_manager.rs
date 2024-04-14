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

use crate::cluster::router::condition::{
    condition_router::{ConditionRouter, ConditionSingleRouters},
    single_router::ConditionSingleRouter,
};
use crate::config::router::ConditionRouterConfig;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone, Default)]
pub struct ConditionRouterManager {
    //Application-level routing applies globally, while service-level routing only affects a specific service.
    pub routers_service: HashMap<String, Arc<RwLock<ConditionSingleRouters>>>,
    pub routers_application: Arc<RwLock<ConditionSingleRouters>>,
}

impl ConditionRouterManager {
    pub fn get_router(&self, service_name: &String) -> Option<ConditionRouter> {
        let routers_application_is_null = self.routers_application.read().unwrap().is_null();
        self.routers_service
            .get(service_name)
            .map(|routers_service| {
                ConditionRouter::new(
                    Some(routers_service.clone()),
                    if routers_application_is_null {
                        None
                    } else {
                        Some(self.routers_application.clone())
                    },
                )
            })
            .or_else(|| {
                if routers_application_is_null {
                    None
                } else {
                    Some(ConditionRouter::new(
                        None,
                        Some(self.routers_application.clone()),
                    ))
                }
            })
    }

    pub fn update(&mut self, config: ConditionRouterConfig) {
        let force = config.force;
        let scope = config.scope;
        let key = config.key;
        let enable = config.enabled;

        let routers = config
            .conditions
            .into_iter()
            .map(|condition| ConditionSingleRouter::new(condition, force, enable))
            .collect::<Vec<_>>();

        match scope.as_str() {
            "application" => {
                self.routers_application.write().unwrap().routers = routers;
            }
            "service" => {
                self.routers_service
                    .entry(key)
                    .or_insert_with(|| Arc::new(RwLock::new(ConditionSingleRouters::new(vec![]))))
                    .write()
                    .unwrap()
                    .routers = routers;
            }
            _ => {}
        }
    }
}
