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

use crate::Level;

pub(crate) fn default() {
    if let Some(true) = configured() {
        parse_from_config()
    } else {
        tracing_subscriber::fmt()
            .compact()
            // enable everything
            .with_max_level(Level::TRACE)
            .with_thread_names(false)
            .with_line_number(true)
            // sets this to be the default, global collector for this application.
            .init();
    }
}

pub(crate) fn parse_from_config() {
    todo!()
}

pub(crate) fn configured() -> Option<bool> {
    None
}
