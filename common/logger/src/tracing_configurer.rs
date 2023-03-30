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

// https://github.com/tokio-rs/tracing/issues/971

use crate::level::LevelWrapper;
use dubbo_utils::{path_util, yaml_util};
use std::path::PathBuf;
use tracing::debug;

pub(crate) fn default() {
    let path_buf = PathBuf::new()
        .join(path_util::app_root_dir())
        .join("application.yaml");
    let level: LevelWrapper = yaml_util::yaml_key_reader(path_buf, "logging.level")
        .unwrap()
        .into();
    tracing_subscriber::fmt()
        .compact()
        .with_line_number(true)
        .with_max_level(level.inner)
        // sets this to be the default, global collector for this application.
        .try_init()
        .expect("init err.");
    debug!("Tracing configured.")
}
