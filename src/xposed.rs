/**
 * xposed.rs
 * Password data breach check implementation.
 *
 * Only the first 10 hex characters of the password's keccak-512 hash are
 * sent to XposedOrNot (k-anonymity), never the password itself.
 */
use std::fmt::Write as FmtWrite;
use std::time::Duration;

use isahc::config::Configurable;
use isahc::prelude::*;
use isahc::Request;
use serde_json::Value;
use sha3::{Digest, Keccak512};
use zeroize::Zeroizing;

use crate::Result;

const API_ENDPOINT: &str = "https://passwords.xposedornot.com/api/v1/pass/anon/";
const TIMEOUT: Duration = Duration::from_secs(30);

fn hash_prefix(password: &str) -> String {
    let mut hasher = Keccak512::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();

    // converting hash byte values to hex
    let mut hash_in_hex = Zeroizing::new(String::new());
    for b in result {
        write!(hash_in_hex, "{:02x}", b).unwrap();
    }

    hash_in_hex[0..10].to_string()
}

/// Extracts the breach count from an API response, None if the password was not found.
fn parse_count(data: &Value) -> Result<Option<u64>> {
    let search = match data.get("SearchPassAnon") {
        Some(search) => search,
        None => return Ok(None),
    };

    let count = match search.get("count") {
        Some(Value::String(s)) => s.parse().ok(),
        Some(Value::Number(n)) => n.as_u64(),
        _ => None,
    };
    count
        .map(Some)
        .ok_or_else(|| "Unexpected response from XposedOrNot.".into())
}

pub fn xposed(password: &str) -> Result<()> {
    let api_endpoint = format!("{}{}", API_ENDPOINT, hash_prefix(password));

    let mut response = Request::get(api_endpoint)
        .timeout(TIMEOUT)
        .body(())?
        .send()
        .map_err(|why| format!("Cannot send request to XposedOrNot: {}", why))?;
    let response_text = response
        .text()
        .map_err(|why| format!("Cannot read response from XposedOrNot: {}", why))?;

    let status = response.status();
    if !status.is_success() && status.as_u16() != 404 {
        return Err(format!("XposedOrNot returned an error: {}", status).into());
    }

    // a 404 response means the hash was not found in any breach.
    let count = match serde_json::from_str::<Value>(&response_text) {
        Ok(data) => parse_count(&data)?,
        Err(_) if status.as_u16() == 404 => None,
        Err(_) => return Err("Unexpected response from XposedOrNot.".into()),
    };

    match count {
        Some(count) if count > 0 => println!(
            "Password is leaked and observed {} times in xposedornot data breaches.
It is advised to change your password for this domain with 'update' command,
and generate a random password using 'g' option when prompted.",
            count
        ),
        _ => println!("Password not found in xposedornot data breaches."),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn only_a_short_hash_prefix_is_sent() {
        let prefix = hash_prefix("password");
        assert_eq!(prefix.len(), 10);
        assert!(!prefix.contains("password"));
    }

    #[test]
    fn parses_api_responses() {
        let found = json!({"SearchPassAnon": {"anon": "x", "count": "42", "wordlist": 0}});
        assert_eq!(parse_count(&found).unwrap(), Some(42));
        let numeric = json!({"SearchPassAnon": {"count": 7}});
        assert_eq!(parse_count(&numeric).unwrap(), Some(7));
        let not_found = json!({"Error": "Not found"});
        assert_eq!(parse_count(&not_found).unwrap(), None);
        let malformed = json!({"SearchPassAnon": {"count": "lots"}});
        assert!(parse_count(&malformed).is_err());
    }
}
