use std::env;

pub fn get_env_value(env_key: &str) -> Option<String> {
    env::var(env_key).ok()
}

pub fn get_env_value_for_i32(env_key: &str) -> Option<i32> {
    get_env_value(env_key).map(|v| v.parse::<i32>().unwrap())
}

#[cfg(test)]
mod tests {
    use crate::env_util::{get_env_value, get_env_value_for_i32};
    use std::env;

    #[test]
    fn test_get_env_value() {
        env::set_var("TEST_ENV", "testxxx1");
        env::set_var("TEST_ENV3", "999");
        assert!(get_env_value("TEST_ENV").is_some());
        assert!(get_env_value("TEST_ENV2").is_none());
        assert_eq!(get_env_value_for_i32("TEST_ENV3"), Some(999_i32))
    }
}
