use dubbo_base::Url;
use dubbo_logger::tracing::info;
use regex::Regex;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    cluster::router::{condition::matcher::ConditionMatcher, utils::to_original_map, Router},
    codegen::RpcInvocation,
    invocation::Invocation,
};

#[derive(Debug, Clone, Default)]
pub struct ConditionSingleRouter {
    pub name: String,
    pub when_condition: HashMap<String, Arc<RwLock<ConditionMatcher>>>,
    pub then_condition: HashMap<String, Arc<RwLock<ConditionMatcher>>>,
    pub enabled: bool,
    pub force: bool,
}

impl Router for ConditionSingleRouter {
    fn route(&self, invokers: Vec<Url>, url: Url, invocation: Arc<RpcInvocation>) -> Vec<Url> {
        if !self.enabled {
            return invokers;
        };
        let mut result = Vec::new();
        if self.match_when(url.clone(), invocation.clone()) {
            for invoker in &invokers.clone() {
                if self.match_then(invoker.clone(), invocation.clone()) {
                    result.push(invoker.clone());
                }
            }
            if result.is_empty() && self.force == false {
                invokers
            } else {
                result
            }
        } else {
            invokers
        }
    }
}

impl ConditionSingleRouter {
    pub fn new(rule: String, force: bool, enabled: bool) -> Self {
        let mut router = Self {
            name: "condition".to_string(),
            when_condition: HashMap::new(),
            then_condition: HashMap::new(),
            enabled,
            force,
        };
        if enabled {
            router.init(rule).expect("parse rule error");
        }
        router
    }

    fn init(&mut self, rule: String) -> Result<(), Box<dyn std::error::Error>> {
        match rule.trim().is_empty() {
            true => Err("Illegal route rule!".into()),
            false => {
                let r = rule.replace("consumer.", "").replace("provider.", "");
                let i = r.find("=>").unwrap_or_else(|| r.len());
                let when_rule = r[..i].trim().to_string();
                let then_rule = r[(i + 2)..].trim().to_string();
                let when = if when_rule.is_empty() || when_rule == "true" {
                    HashMap::new()
                } else {
                    self.parse_rule(&when_rule)?
                };
                let then = if then_rule.is_empty() || then_rule == "false" {
                    HashMap::new()
                } else {
                    self.parse_rule(&then_rule)?
                };
                self.when_condition = when;
                self.then_condition = then;
                Ok(())
            }
        }
    }

    fn parse_rule(
        &mut self,
        rule: &str,
    ) -> Result<HashMap<String, Arc<RwLock<ConditionMatcher>>>, Box<dyn std::error::Error>> {
        let mut conditions: HashMap<String, Arc<RwLock<ConditionMatcher>>> = HashMap::new();
        let mut current_matcher: Option<Arc<RwLock<ConditionMatcher>>> = None;
        let regex = Regex::new(r"([&!=,]*)\s*([^&!=,\s]+)").unwrap();
        for cap in regex.captures_iter(rule) {
            let separator = &cap[1];
            let content = &cap[2];

            match separator {
                "" => {
                    let current_key = content.to_string();
                    current_matcher =
                        Some(Arc::new(RwLock::new(self.get_matcher(current_key.clone()))));
                    conditions.insert(
                        current_key.clone(),
                        current_matcher.as_ref().unwrap().clone(),
                    );
                }
                "&" => {
                    let current_key = content.to_string();
                    current_matcher = conditions.get(&current_key).cloned();
                    if current_matcher.is_none() {
                        let matcher = Arc::new(RwLock::new(self.get_matcher(current_key.clone())));
                        conditions.insert(current_key.clone(), matcher.clone());
                        current_matcher = Some(matcher);
                    }
                }
                "=" => {
                    if let Some(matcher) = &current_matcher {
                        let mut matcher_borrowed = matcher.write().unwrap();
                        matcher_borrowed
                            .get_matches()
                            .insert(content.to_string().clone());
                    } else {
                        return Err("Error: ...".into());
                    }
                }
                "!=" => {
                    if let Some(matcher) = &current_matcher {
                        let mut matcher_borrowed = matcher.write().unwrap();
                        matcher_borrowed
                            .get_mismatches()
                            .insert(content.to_string().clone());
                    } else {
                        return Err("Error: ...".into());
                    }
                }
                "," => {
                    if let Some(matcher) = &current_matcher {
                        let mut matcher_borrowed = matcher.write().unwrap();
                        if matcher_borrowed.get_matches().is_empty()
                            && matcher_borrowed.get_mismatches().is_empty()
                        {
                            return Err("Error: ...".into());
                        }
                        drop(matcher_borrowed);
                        let mut matcher_borrowed_mut = matcher.write().unwrap();
                        matcher_borrowed_mut
                            .get_matches()
                            .insert(content.to_string().clone());
                    } else {
                        return Err("Error: ...".into());
                    }
                }
                _ => {
                    return Err("Error: ...".into());
                }
            }
        }
        Ok(conditions)
    }

    // pub fn is_runtime(&self) -> bool {
    //     // same as the Java version
    // }

    pub fn get_matcher(&self, key: String) -> ConditionMatcher {
        ConditionMatcher::new(key)
    }

    pub fn match_when(&self, url: Url, invocation: Arc<RpcInvocation>) -> bool {
        if self.when_condition.is_empty() {
            return true;
        }
        self.do_match(url, &self.when_condition, invocation)
    }

    pub fn match_then(&self, url: Url, invocation: Arc<RpcInvocation>) -> bool {
        if self.then_condition.is_empty() {
            return false;
        }
        self.do_match(url, &self.then_condition, invocation)
    }

    pub fn do_match(
        &self,
        url: Url,
        conditions: &HashMap<String, Arc<RwLock<ConditionMatcher>>>,
        invocation: Arc<RpcInvocation>,
    ) -> bool {
        let sample: HashMap<String, String> = to_original_map(url);
        conditions.iter().all(|(key, condition_matcher)| {
            let matcher = condition_matcher.read().unwrap();
            let value = get_value(key, &sample, invocation.clone());
            match matcher.is_match(value) {
                Ok(result) => result,
                Err(error) => {
                    info!("Error occurred: {:?}", error);
                    false
                }
            }
        })
    }
}

fn get_value(
    key: &String,
    sample: &HashMap<String, String>,
    invocation: Arc<RpcInvocation>,
) -> Option<String> {
    if key == "method" {
        let method_param = invocation.get_method_name();
        return Some(method_param);
    }
    sample.get(key).cloned()
}
