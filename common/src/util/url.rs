
use url::Url;

use crate::util::hash::hash_to_u64;

pub fn to_identity_string_with_url<'a>(url: Url) -> String {
    let url_string = url.to_string();
    hash_to_u64(&url_string).to_string()
}

pub fn to_identity_string_with_url_and_method_name(url: Url, method_name: &str) -> String {
    let url_string = url.to_string();
    hash_to_u64(&url_string).to_string() + ":" + method_name
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use url::Url;

    use crate::util::url::to_identity_string;

    #[test]
    fn test_to_identity_string() {
        let url = Url::from_str("https://www.taobao.com/wd?tx=1");
        let string = to_identity_string(url.unwrap());
        println!("{}", string);
    }
}