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

use std::sync::atomic::{AtomicBool, Ordering};

pub use tracing::{self, Level};

// flag for the sake of avoiding multiple initialization
static TRACING_CONFIGURED: AtomicBool = AtomicBool::new(false);

mod level;
mod tracing_configurer;

// put on main method
pub fn init() {
    if !TRACING_CONFIGURED.load(Ordering::SeqCst) {
        tracing_configurer::default()
    }
}

#[cfg(test)]
mod tests {
    use tracing::{debug, info, trace};

    #[test]
    fn test_print_info_log() {
        super::init();
        debug!("hello rust:debug");
        trace!("hello rust:trace");
        info!("hello rust:info");
    }
}
