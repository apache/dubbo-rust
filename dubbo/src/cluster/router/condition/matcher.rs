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

use regex::Regex;
use std::{collections::HashSet, error::Error, option::Option};

#[derive(Clone, Debug, Default)]
pub struct ConditionMatcher {
    _key: String,
    matches: HashSet<String>,
    mismatches: HashSet<String>,
}

impl ConditionMatcher {
    pub fn new(_key: String) -> Self {
        ConditionMatcher {
            _key,
            matches: HashSet::new(),
            mismatches: HashSet::new(),
        }
    }

    pub fn is_match(&self, value: Option<String>) -> Result<bool, Box<dyn Error>> {
        match value {
            None => Ok(false),
            Some(val) => {
                for match_ in self.matches.iter() {
                    if self.do_pattern_match(match_, &val) {
                        return Ok(true);
                    }
                }
                for mismatch in self.mismatches.iter() {
                    if !self.do_pattern_match(mismatch, &val) {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }

    pub fn get_matches(&mut self) -> &mut HashSet<String> {
        &mut self.matches
    }
    pub fn get_mismatches(&mut self) -> &mut HashSet<String> {
        &mut self.mismatches
    }

    fn do_pattern_match(&self, pattern: &str, value: &str) -> bool {
        if pattern.contains('*') {
            return star_matcher(pattern, value);
        }

        if pattern.contains('~') {
            let parts: Vec<&str> = pattern.split('~').collect();

            if parts.len() == 2 {
                if let (Ok(left), Ok(right), Ok(val)) = (
                    parts[0].parse::<i32>(),
                    parts[1].parse::<i32>(),
                    value.parse::<i32>(),
                ) {
                    return range_matcher(val, left, right);
                }
            }
            return false;
        }
        pattern == value
    }
}

pub fn star_matcher(pattern: &str, input: &str) -> bool {
    // 将*替换为任意字符的正则表达式
    let pattern = pattern.replace("*", ".*");
    let regex = Regex::new(&pattern).unwrap();
    regex.is_match(input)
}

pub fn range_matcher(val: i32, min: i32, max: i32) -> bool {
    min <= val && val <= max
}
