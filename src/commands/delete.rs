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

use crate::config::encode;

pub fn run(
    base_url: &str,
    token: &str,
    name: &str,
    gateway: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut url = format!("{base_url}/api/workspaces/{}", encode(name));
    if let Some(gw) = gateway {
        url = format!("{url}?gateway={}", encode(gw));
    }

    ureq::delete(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| format!("Failed to delete workspace: {e}"))?;

    println!("Workspace \"{name}\" deleted.");
    Ok(())
}
