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

use comfy_table::{Cell, Table};
use serde::Deserialize;

#[derive(Deserialize)]
struct Workspace {
    id: String,
    name: String,
    phase: String,
    source_path: Option<String>,
    gateway: String,
}

pub fn run(
    base_url: &str,
    token: &str,
    json_output: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{base_url}/api/workspaces");
    let body: String = ureq::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| format!("Cannot reach Kaiden. Is the application running? ({e})"))?
        .body_mut()
        .read_to_string()?;

    if json_output {
        println!("{body}");
        return Ok(());
    }

    let workspaces: Vec<Workspace> = serde_json::from_str(&body)?;

    let mut table = Table::new();
    table.set_header(vec!["NAME", "ID", "PHASE", "GATEWAY", "SOURCE"]);

    for ws in &workspaces {
        table.add_row(vec![
            Cell::new(&ws.name),
            Cell::new(&ws.id[..ws.id.len().min(12)]),
            Cell::new(&ws.phase),
            Cell::new(&ws.gateway),
            Cell::new(ws.source_path.as_deref().unwrap_or("")),
        ]);
    }

    println!("{table}");
    Ok(())
}
