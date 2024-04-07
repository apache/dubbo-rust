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
use super::Level;

pub(crate) struct LevelWrapper {
    pub(crate) inner: Level,
}
impl LevelWrapper {
    pub fn new(level: Level) -> Self {
        LevelWrapper { inner: level }
    }
}

impl From<Option<String>> for LevelWrapper {
    fn from(s: Option<String>) -> Self {
        match s.unwrap().to_lowercase().as_str().trim() {
            "error" => LevelWrapper::new(Level::ERROR),
            "warn" => LevelWrapper::new(Level::WARN),
            "info" => LevelWrapper::new(Level::INFO),
            "debug" => LevelWrapper::new(Level::DEBUG),
            "trace" => LevelWrapper::new(Level::TRACE),
            _ => LevelWrapper::new(Level::INFO),
        }
    }
}
