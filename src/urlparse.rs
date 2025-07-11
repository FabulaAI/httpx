use std::net::{Ipv4Addr, Ipv6Addr};

use num_bigint::BigInt;
use pyo3::{prelude::*, types::PyString};

use crate::err::InvalidUrl;

#[pyfunction]
pub fn normalize_path(path: &str) -> String {
    if !path.contains(".") {
        return path.to_owned();
    }

    let components = path.split('/').collect::<Vec<&str>>();
    let mut normalized_components = Vec::with_capacity(components.len());

    for component in components {
        if component == "." {
            continue;
        } else if component == ".." {
            if !normalized_components.is_empty() && (&normalized_components != &[""]) {
                normalized_components.pop();
            }
        } else {
            normalized_components.push(component);
        }
    }

    normalized_components.join("/")
}

const UNRESERVED_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";

pub fn percent_encoded(string: &str, safe: &str) -> String {
    let safe = safe.as_bytes();
    string
        .bytes()
        .map(|b| {
            if UNRESERVED_CHARS.contains(&b) || safe.contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{:02X}", b)
            }
        })
        .collect::<String>()
}

fn is_percent_encoded(s: &[u8]) -> bool {
    s.len() == 3 && s[0] == b'%' && s[1].is_ascii_hexdigit() && s[2].is_ascii_hexdigit()
}

#[pyfunction]
pub fn quote(string: &str, safe: &str) -> String {
    let s = string.as_bytes();
    let mut result = String::with_capacity(s.len());

    let mut start = 0;
    let mut i = 0;
    while i < s.len() {
        if s[i] == b'%' && i + 2 < s.len() && is_percent_encoded(&s[i..i + 3]) {
            if start < i {
                result.push_str(&percent_encoded(&string[start..i], safe));
            }
            result.push_str(&string[i..i + 3]);
            i += 3;
            start = i;
        } else {
            i += 1;
        }
    }

    if start < s.len() {
        result.push_str(&percent_encoded(&string[start..], safe));
    }

    result
}

#[pyfunction]
pub fn find_ascii_non_printable(s: &str) -> Option<usize> {
    s.chars()
        .position(|c| c.is_ascii() && !c.is_ascii_graphic() && c != ' ')
}

pub(crate) trait PercentEncoded {
    fn percent_encoded(&self, safe: &str) -> String;
}

impl PercentEncoded for String {
    fn percent_encoded(&self, safe: &str) -> String {
        quote(self, safe)
    }
}

impl PercentEncoded for &str {
    fn percent_encoded(&self, safe: &str) -> String {
        quote(self, safe)
    }
}

#[pyfunction]
pub fn validate_path(path: &str, has_scheme: bool, has_authority: bool) -> PyResult<()> {
    if has_authority && !path.is_empty() && !path.starts_with('/') {
        return Err(InvalidUrl::new("For absolute URLs, path must be empty or begin with '/'").into());
    }

    if !has_scheme && !has_authority {
        if path.starts_with("//") {
            return Err(InvalidUrl::new("Relative URLs cannot have a path starting with '//'").into());
        }
        if path.starts_with(':') {
            return Err(InvalidUrl::new("Relative URLs cannot have a path starting with ':'").into());
        }
    }

    Ok(())
}

#[pyfunction]
pub fn normalize_port(port: &Bound<'_, PyAny>, scheme: &str) -> PyResult<Option<BigInt>> {
    if port.is_none() {
        return Ok(None);
    }

    let port = if port.is_instance_of::<PyString>() {
        let port_str = port.extract::<&str>()?;
        if port_str.is_empty() {
            return Ok(None);
        }
        match port_str.parse::<BigInt>() {
            Ok(p) => p,
            Err(_) => return Err(InvalidUrl::new(&format!("Invalid port: '{}'", port_str)).into()),
        }
    } else {
        match port.extract::<BigInt>() {
            Ok(p) => p,
            Err(_) => return Err(InvalidUrl::new(&format!("Invalid port: {}", port.repr()?)).into()),
        }
    };

    if let Some(default_port) = match scheme {
        "https" | "wss" => Some(BigInt::from(443)),
        "http" | "ws" => Some(BigInt::from(80)),
        "ftp" => Some(BigInt::from(21)),
        _ => None,
    } {
        if port == default_port {
            return Ok(None);
        }
    }

    Ok(Some(port))
}

fn is_ip_v4_like(s: &str) -> bool {
    regex::Regex::new(r"^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$")
        .unwrap()
        .is_match(s)
}

fn is_ip_v6_like(s: &str) -> bool {
    regex::Regex::new(r"^\[.*\]$").unwrap().is_match(s)
}

fn encode_idna(host: &str) -> PyResult<String> {
    Python::with_gil(|py| {
        let idna = PyModule::import(py, "idna")?;
        let host_str = PyString::new(py, host);
        String::from_utf8(
            idna.call_method1("encode", (host_str,))
                .map_err(|_| InvalidUrl::new(&format!("Invalid IDNA hostname: '{}'", host)))?
                .extract::<Vec<u8>>()?,
        )
        .map_err(|e| e.into())
    })
}

#[pyfunction]
pub fn encode_host(host: &str) -> PyResult<String> {
    if host.is_empty() {
        return Ok(String::new());
    }

    if is_ip_v4_like(host) {
        match host.parse::<Ipv4Addr>() {
            Ok(_) => return Ok(host.to_owned()),
            Err(_) => return Err(InvalidUrl::new(&format!("Invalid IPv4 address: '{}'", host)).into()),
        }
    }

    if is_ip_v6_like(host) {
        let ip = host.trim_matches(|c| c == '[' || c == ']');
        match ip.parse::<Ipv6Addr>() {
            Ok(_) => return Ok(host.to_owned()),
            Err(_) => return Err(InvalidUrl::new(&format!("Invalid IPv6 address: '{}'", host)).into()),
        }
    }

    if host.is_ascii() {
        return Ok(host
            .to_ascii_lowercase()
            .percent_encoded("!$&'()*+,;=\"`{}%|\\"));
    }

    encode_idna(&host.to_lowercase())
}
