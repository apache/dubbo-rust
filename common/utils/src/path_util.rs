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

use std::{env, path::PathBuf};

pub fn app_root_dir() -> PathBuf {
    match project_root::get_project_root() {
        // Cargo.lock file as app root dir
        Ok(p) => p,
        Err(_) => env::current_dir().unwrap(),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_app_root_dir() {
        let dir = app_root_dir().join("application.yaml");
        println!("dir: {}", dir.display());
    }
}
