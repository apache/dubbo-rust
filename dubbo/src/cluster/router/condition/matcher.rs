use crate::codegen::RpcInvocation;
use regex::Regex;
use std::{collections::HashSet, error::Error, option::Option, sync::Arc};

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

    pub fn is_match(
        &self,
        value: Option<String>,
        invocation: Arc<RpcInvocation>,
        _is_when: bool,
    ) -> Result<bool, Box<dyn Error>> {
        match value {
            None => {
                // if key does not present in whichever of url, invocation or attachment based on the matcher type, then return false.
                Ok(false)
            }
            Some(val) => {
                if !self.matches.is_empty() && self.mismatches.is_empty() {
                    for match_ in self.matches.iter() {
                        if self.do_pattern_match(match_, &val, invocation.clone())? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                } else if !self.mismatches.is_empty() && self.matches.is_empty() {
                    for mismatch in self.mismatches.iter() {
                        if self.do_pattern_match(mismatch, &val, invocation.clone())? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else if !self.matches.is_empty() && !self.mismatches.is_empty() {
                    for mismatch in self.mismatches.iter() {
                        if self.do_pattern_match(mismatch, &val, invocation.clone())? {
                            return Ok(false);
                        }
                    }
                    for match_ in self.matches.iter() {
                        if self.do_pattern_match(match_, &val, invocation.clone())? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                } else {
                    Ok(false)
                }
            }
        }
    }
    pub fn get_matches(&mut self) -> &mut HashSet<String> {
        &mut self.matches
    }
    pub fn get_mismatches(&mut self) -> &mut HashSet<String> {
        &mut self.mismatches
    }

    fn do_pattern_match(
        &self,
        pattern: &String,
        value: &String,
        _invocation: Arc<RpcInvocation>,
    ) -> Result<bool, Box<dyn Error>> {
        if pattern.contains("*") {
            return Ok(star_matcher(pattern, value));
        }
        Ok(pattern.eq(value))
    }
}

pub fn star_matcher(pattern: &str, input: &str) -> bool {
    // 将*替换为任意字符的正则表达式
    let pattern = pattern.replace("*", ".*");
    let regex = Regex::new(&pattern).unwrap();
    regex.is_match(input)
}

pub fn _range_matcher(val: i32, min: i32, max: i32) -> bool {
    min <= val && val <= max
}
