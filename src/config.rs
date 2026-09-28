/**********************************************************************
 * Copyright (C) 2026 Red Hat, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 ***********************************************************************/

use std::fs;
use std::path::PathBuf;

// Kaiden writes the port file in its data directory: KAIDEN_HOME_DIR (legacy layout),
// $XDG_DATA_HOME/kaiden (Linux XDG layout) or ~/.local/share/kaiden (default for both)
fn port_file_candidates() -> Vec<PathBuf> {
    let mut data_dirs = Vec::new();
    if let Some(dir) = std::env::var_os("KAIDEN_HOME_DIR") {
        data_dirs.push(PathBuf::from(dir));
    }
    if let Some(dir) = std::env::var_os("XDG_DATA_HOME") {
        data_dirs.push(PathBuf::from(dir).join("kaiden"));
    }
    if let Some(home) = dirs::home_dir() {
        data_dirs.push(home.join(".local").join("share").join("kaiden"));
    }
    data_dirs
        .into_iter()
        .map(|dir| dir.join("api-port"))
        .collect()
}

fn parse_port_file(content: &str) -> Option<(u16, String)> {
    let mut lines = content.lines();
    let port = lines.next()?.trim().parse().ok()?;
    let token = lines.next()?.trim();
    (!token.is_empty()).then(|| (port, token.to_string()))
}

/// Returns the API base URL and the bearer token, both published by Kaiden in the port file.
pub fn api_endpoint() -> Result<(String, String), Box<dyn std::error::Error>> {
    let (port_file, content) = port_file_candidates()
        .into_iter()
        .find_map(|path| fs::read_to_string(&path).ok().map(|content| (path, content)))
        .ok_or("Kaiden API is not available. Start Kaiden (and check that the 'api.server.enabled' preference is on).")?;
    let (port, token) = parse_port_file(&content)
        .ok_or_else(|| format!("Invalid content in {}", port_file.display()))?;
    Ok((format!("http://127.0.0.1:{port}"), token))
}

/// Percent-encodes a value for use in a URL path segment or query value.
pub fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_port_and_token() {
        assert_eq!(
            parse_port_file("4242\nabc\n"),
            Some((4242, "abc".to_string()))
        );
    }

    #[test]
    fn rejects_incomplete_port_file() {
        assert_eq!(parse_port_file("4242\n"), None);
        assert_eq!(parse_port_file("4242\n\n"), None);
        assert_eq!(parse_port_file("nope\nabc\n"), None);
    }

    #[test]
    fn encodes_reserved_characters() {
        assert_eq!(encode("ws-1_a.b~"), "ws-1_a.b~");
        assert_eq!(encode("x&agent=true"), "x%26agent%3Dtrue");
        assert_eq!(encode("a/b c"), "a%2Fb%20c");
    }
}
