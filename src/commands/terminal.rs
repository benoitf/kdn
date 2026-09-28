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

use std::io::{Read, Write};
use std::sync::mpsc;

use rustix::termios::{self, OptionalActions, Termios};
use tungstenite::Message;
use tungstenite::client::IntoClientRequest;

use crate::config::encode;

pub fn run(
    base_url: &str,
    token: &str,
    sandbox_name: &str,
    command: &[String],
    gateway: Option<&str>,
) -> Result<i32, Box<dyn std::error::Error>> {
    run_terminal(base_url, token, sandbox_name, command, gateway, false)
}

/// WebSocket URL of the workspace PTY endpoint, with the optional gateway and agent query parameters.
fn pty_url(base_url: &str, sandbox_name: &str, gateway: Option<&str>, agent: bool) -> String {
    let ws_url = base_url.replacen("http://", "ws://", 1);
    let mut url = format!("{ws_url}/api/workspaces/{}/pty", encode(sandbox_name));
    let mut params = Vec::new();
    if let Some(gw) = gateway {
        params.push(format!("gateway={}", encode(gw)));
    }
    if agent {
        params.push("agent=true".to_string());
    }
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }
    url
}

pub fn run_terminal(
    base_url: &str,
    token: &str,
    sandbox_name: &str,
    command: &[String],
    gateway: Option<&str>,
    agent: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    let mut request = pty_url(base_url, sandbox_name, gateway, agent).into_client_request()?;
    request
        .headers_mut()
        .insert("Authorization", format!("Bearer {token}").parse()?);
    let (mut ws, _) = tungstenite::connect(request)?;

    let cmd: Vec<&str> = if command.is_empty() {
        vec!["/bin/sh"]
    } else {
        command.iter().map(String::as_str).collect()
    };

    let (cols, rows) = term_size()?;
    let start_msg = serde_json::json!({
        "command": cmd,
        "cols": cols,
        "rows": rows,
    });
    ws.send(Message::Text(start_msg.to_string().into()))?;

    let _raw_guard = RawModeGuard::enable()?;

    // short read timeout instead of a non-blocking socket: the loop idles in read() rather than spinning,
    // while stdin and resizes are still picked up within one tick
    if let tungstenite::stream::MaybeTlsStream::Plain(s) = ws.get_ref() {
        s.set_read_timeout(Some(std::time::Duration::from_millis(10)))?;
    }

    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    std::thread::spawn(move || {
        let mut stdin = std::io::stdin().lock();
        let mut buf = [0u8; 4096];
        loop {
            match stdin.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => break,
            }
        }
    });

    let mut stdout = std::io::stdout();
    let mut last_size = (cols, rows);

    loop {
        while let Ok(bytes) = rx.try_recv() {
            if ws.send(Message::Binary(bytes.into())).is_err() {
                return Ok(0);
            }
        }

        if let Ok(size) = term_size()
            && size != last_size
        {
            last_size = size;
            let msg = serde_json::json!({
                "type": "resize",
                "cols": size.0,
                "rows": size.1,
            });
            let _ = ws.send(Message::Text(msg.to_string().into()));
        }

        loop {
            match ws.read() {
                Ok(Message::Binary(data)) => {
                    stdout.write_all(&data)?;
                    stdout.flush()?;
                }
                Ok(Message::Text(text)) => {
                    if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&text) {
                        match msg.get("type").and_then(|t| t.as_str()) {
                            Some("exit") => {
                                let code = msg
                                    .get("exitCode")
                                    .and_then(serde_json::Value::as_i64)
                                    .unwrap_or(0);
                                return Ok(i32::try_from(code).unwrap_or(1));
                            }
                            Some("error") => {
                                let err = msg
                                    .get("error")
                                    .and_then(|e| e.as_str())
                                    .unwrap_or("unknown");
                                return Err(err.to_string().into());
                            }
                            _ => {}
                        }
                    }
                }
                Err(tungstenite::Error::Io(ref e))
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    break;
                }
                Ok(Message::Close(_)) | Err(_) => return Ok(0),
                Ok(_) => {}
            }
        }
    }
}

fn term_size() -> Result<(u16, u16), Box<dyn std::error::Error>> {
    let ws = termios::tcgetwinsize(std::io::stdout())
        .map_err(|e| format!("failed to get terminal size: {e}"))?;
    Ok((ws.ws_col, ws.ws_row))
}

/// Puts stdin in raw mode and restores the original settings when dropped.
struct RawModeGuard(Termios);

impl RawModeGuard {
    fn enable() -> Result<Self, Box<dyn std::error::Error>> {
        let stdin = std::io::stdin();
        let original = termios::tcgetattr(&stdin).map_err(|e| format!("tcgetattr failed: {e}"))?;
        let mut raw = original.clone();
        raw.make_raw();
        termios::tcsetattr(&stdin, OptionalActions::Flush, &raw)
            .map_err(|e| format!("tcsetattr failed: {e}"))?;
        Ok(Self(original))
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = termios::tcsetattr(std::io::stdin(), OptionalActions::Flush, &self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pty_url_without_params() {
        assert_eq!(
            pty_url("http://127.0.0.1:4242", "ws-1", None, false),
            "ws://127.0.0.1:4242/api/workspaces/ws-1/pty"
        );
    }

    #[test]
    fn pty_url_with_gateway() {
        assert_eq!(
            pty_url("http://127.0.0.1:4242", "ws-1", Some("gw a"), false),
            "ws://127.0.0.1:4242/api/workspaces/ws-1/pty?gateway=gw%20a"
        );
    }

    #[test]
    fn pty_url_with_agent() {
        assert_eq!(
            pty_url("http://127.0.0.1:4242", "ws-1", None, true),
            "ws://127.0.0.1:4242/api/workspaces/ws-1/pty?agent=true"
        );
    }

    #[test]
    fn pty_url_with_gateway_and_agent_encodes_name() {
        assert_eq!(
            pty_url("http://127.0.0.1:4242", "a/b", Some("gw"), true),
            "ws://127.0.0.1:4242/api/workspaces/a%2Fb/pty?gateway=gw&agent=true"
        );
    }
}
