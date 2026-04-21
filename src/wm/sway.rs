//! # Sway IPC Tree Structure
//!
//! `get_sway_tree()` returns the full node tree from sway's `GET_TREE` IPC message.
//! The tree is a recursive structure: `root` → `output` → `workspace` → `con` (windows).
//!
//! ## Node types (`NodeType`)
//!
//! | Type          | Description                                           |
//! |---------------|-------------------------------------------------------|
//! | `Root`        | Single top-level node; contains outputs               |
//! | `Output`      | A physical display (e.g. `eDP-1`, `HDMI-A-1`)        |
//! | `Workspace`   | A numbered/named workspace on an output               |
//! | `Con`         | A tiling or floating container; leaf cons hold a PID  |
//! | `FloatingCon` | A floating window; appears in `floating_nodes`        |
//!
//! ## Relevant fields on a leaf `Con` (a real application window)
//!
//! - `app_id: Option<String>` — Wayland app-id (e.g. `"firefox"`, `"foot"`). Preferred name.
//! - `name: Option<String>` — Window title. Fallback when `app_id` is absent (XWayland apps).
//! - `pid: Option<i32>` — Process ID of the application; `None` on pure containers.
//! - `shell: Option<String>` — `"xdg_shell"` (Wayland) or `"xwayland"`.
//! - `focused: bool` — Whether this node holds current keyboard focus.
//! - `rect` — Absolute screen coordinates `{x, y, width, height}`.
//! - `nodes: Vec<Node>` — Tiling children (empty for leaf windows).
//! - `floating_nodes: Vec<Node>` — Floating children.
//!
//! ## Abbreviated example (`swaymsg -t get_tree`)
//!
//! ```json
//! {
//!   "id": 1,
//!   "type": "root",
//!   "name": "root",
//!   "nodes": [
//!     {
//!       "id": 3,
//!       "type": "output",
//!       "name": "eDP-1",
//!       "nodes": [
//!         {
//!           "id": 4,
//!           "type": "workspace",
//!           "name": "1",
//!           "nodes": [
//!             {
//!               "id": 6,
//!               "type": "con",
//!               "name": "foot",          // window title (fallback)
//!               "app_id": "foot",        // Wayland app-id (preferred)
//!               "pid": 12345,
//!               "shell": "xdg_shell",
//!               "focused": true,
//!               "rect": { "x": 0, "y": 30, "width": 960, "height": 1050 },
//!               "nodes": [],
//!               "floating_nodes": []
//!             },
//!             {
//!               "id": 7,
//!               "type": "con",
//!               "name": "nvim — src/main.rs",  // XWayland: only title, no app_id
//!               "app_id": null,
//!               "pid": 12346,
//!               "shell": "xwayland",
//!               "focused": false,
//!               "rect": { "x": 960, "y": 30, "width": 960, "height": 1050 },
//!               "nodes": [],
//!               "floating_nodes": []
//!             }
//!           ],
//!           "floating_nodes": [
//!             {
//!               "id": 8,
//!               "type": "floating_con",
//!               "name": "pavucontrol",
//!               "app_id": "pavucontrol",
//!               "pid": 12347,
//!               "shell": "xdg_shell",
//!               "focused": false,
//!               "rect": { "x": 400, "y": 300, "width": 600, "height": 400 },
//!               "nodes": [],
//!               "floating_nodes": [...]
//!             }
//!           ]
//!         },
//!         {
//!           "id": 9,
//!           "type": "workspace",
//!           "name": "2",
//!           "nodes": [
//!             {
//!               "id": 10,
//!               "type": "con",
//!               "name": "Mozilla Firefox",
//!               "app_id": "firefox",
//!               "pid": 12348,
//!               "shell": "xdg_shell",
//!               "focused": false,
//!               "rect": { "x": 0, "y": 30, "width": 1920, "height": 1050 },
//!               "nodes": [],
//!               "floating_nodes": [...]
//!             }
//!           ],
//!           "floating_nodes": [...]
//!         }
//!       ]
//!     }
//!   ]
//! }
//! ```
//!
//! ## What this module extracts
//!
//! `info()` walks the tree and returns a JSON map of workspace name → app names:
//!
//! ```json
//! {
//!   "1": ["foot", "nvim — src/main.rs", "pavucontrol"],
//!   "2": ["firefox"]
//! }
//! ```
//!
//! App name resolution order per leaf `Con`: `app_id` → `name` (window title).

use anyhow::Context;
use std::collections::HashMap;
use swayipc::{Connection, Node, NodeType};

/// Collect app names from all leaf windows under `node`.
///
/// Descends into both `nodes` (tiling) and `floating_nodes` at every level.
/// Only nodes of type `Con` or `FloatingCon` with a `pid` are treated as real
/// windows. Intermediate split containers (`Con` with no `pid`) are traversed
/// but not collected.
///
/// Name resolution: `app_id` (Wayland) → `name` (window title, XWayland fallback).
fn collect_leaf_apps(node: &Node) -> Vec<String> {
    let mut apps = Vec::new();
    if (node.node_type == NodeType::Con || node.node_type == NodeType::FloatingCon)
        && node.pid.is_some()
    {
        let name = node
            .app_id
            .clone()
            .or_else(|| node.name.clone())
            .unwrap_or_default();
        if !name.is_empty() {
            apps.push(name);
        }
    }
    for child in node.nodes.iter().chain(node.floating_nodes.iter()) {
        apps.extend(collect_leaf_apps(child));
    }
    apps
}

/// Walk the tree from `node`, inserting `workspace_name → [app_names]` into `map`
/// for every non-empty workspace found.
///
/// Stops descending at `Workspace` nodes — delegates leaf collection to
/// [`collect_leaf_apps`]. Workspaces with no visible windows are omitted.
fn collect_ws_and_apps_dfs(node: &Node, map: &mut HashMap<String, Vec<String>>) {
    if node.node_type == NodeType::Workspace {
        let ws_name = node.name.clone().unwrap_or_default();
        let apps: Vec<String> = collect_leaf_apps(node);
        if !apps.is_empty() {
            map.insert(ws_name, apps);
        }
        return;
    }
    for child in node.nodes.iter().chain(node.floating_nodes.iter()) {
        collect_ws_and_apps_dfs(child, map);
    }
}

/// Build a map of `workspace_name → [app_names]` from the sway tree root.
/// Empty workspaces are absent from the returned map.
fn get_ws_and_apps_json(root: &Node) -> HashMap<String, Vec<String>> {
    let mut workspace_apps: HashMap<String, Vec<String>> = HashMap::new();
    collect_ws_and_apps_dfs(&root, &mut workspace_apps);
    workspace_apps
}

/// Connect to the sway IPC socket and return the full layout tree root.
///
/// # Errors
/// Returns an error if the sway IPC socket is unreachable or the tree fetch fails.
fn get_sway_tree() -> Result<Node, anyhow::Error> {
    let mut conn = Connection::new().context("Couldn't connect to sway IPC")?;
    conn.get_tree().context("Couldn't get sway tree")
}

/// Return a pretty-printed JSON object mapping each non-empty workspace name
/// to the list of app names running in it.
///
/// # Errors
/// Propagates errors from [`get_sway_tree`] if the sway IPC socket is unreachable.
pub(crate) fn info() -> anyhow::Result<String> {
    let root = get_sway_tree()?;
    let workspace_apps = get_ws_and_apps_json(&root);
    let json = serde_json::to_string_pretty(&workspace_apps).unwrap();
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Node is #[non_exhaustive] — must build via deserialization.
    //
    // workspace "1"  layout=splith
    //   Two windows split directly inside the workspace — the common case.
    //   Sway makes the workspace itself the split container; no intermediate node.
    //   ├── con  app_id="foot"  pid=1  name="foot title"  → "foot"  (app_id wins over name)
    //   └── con  app_id=null    pid=2  name="nvim"         → "nvim"  (xwayland: falls back to name)
    //
    // workspace "2"  layout=splith
    //   Nested split: user pressed $mod+v inside an already-split workspace, creating
    //   an intermediate con (no pid) that holds two windows in a sub-split.
    //   Floating windows always live in workspace's floating_nodes, never inside a split con.
    //   ├── con  layout=splitv  pid=null  (intermediate sub-split container, skipped as app)
    //   │   ├── con  app_id="kitty"  pid=3                 → "kitty"
    //   │   └── con  app_id="alacritty"  pid=4             → "alacritty"
    //   ├── con  app_id="firefox"  pid=5  name="Mozilla Firefox"  → "firefox"  (app_id wins)
    //   └── floating_nodes
    //       └── floating_con  app_id="pavucontrol"  pid=6  → "pavucontrol"
    //
    // workspace "3"  (empty — must be absent from the result map)
    #[test]
    fn test_recursive_workspace_app_collection() {
        let json = r#"{
          "id":1,"type":"root","name":"root",
          "border":"none","current_border_width":0,"layout":"none","orientation":"none",
          "rect":{"x":0,"y":0,"width":1920,"height":1080},
          "window_rect":{"x":0,"y":0,"width":0,"height":0},
          "deco_rect":{"x":0,"y":0,"width":0,"height":0},
          "geometry":{"x":0,"y":0,"width":0,"height":0},
          "urgent":false,"focused":false,"focus":[],"sticky":false,
          "floating_nodes":[],"marks":[],
          "nodes":[{
            "id":2,"type":"output","name":"eDP-1",
            "border":"none","current_border_width":0,"layout":"output","orientation":"none",
            "rect":{"x":0,"y":0,"width":1920,"height":1080},
            "window_rect":{"x":0,"y":0,"width":0,"height":0},
            "deco_rect":{"x":0,"y":0,"width":0,"height":0},
            "geometry":{"x":0,"y":0,"width":0,"height":0},
            "urgent":false,"focused":false,"focus":[],"sticky":false,
            "floating_nodes":[],"marks":[],
            "nodes":[
              {
                "id":3,"type":"workspace","name":"1",
                "border":"none","current_border_width":0,"layout":"splith","orientation":"horizontal",
                "rect":{"x":0,"y":0,"width":1920,"height":1080},
                "window_rect":{"x":0,"y":0,"width":0,"height":0},
                "deco_rect":{"x":0,"y":0,"width":0,"height":0},
                "geometry":{"x":0,"y":0,"width":0,"height":0},
                "urgent":false,"focused":false,"focus":[],"sticky":false,"marks":[],
                "floating_nodes":[],
                "nodes":[
                  {
                    "id":10,"type":"con","name":"foot title","app_id":"foot","pid":1,"shell":"xdg_shell",
                    "border":"none","current_border_width":0,"layout":"none","orientation":"none",
                    "rect":{"x":0,"y":0,"width":960,"height":1080},
                    "window_rect":{"x":0,"y":0,"width":960,"height":1080},
                    "deco_rect":{"x":0,"y":0,"width":0,"height":0},
                    "geometry":{"x":0,"y":0,"width":960,"height":1080},
                    "urgent":false,"focused":true,"focus":[],"sticky":false,
                    "nodes":[],"floating_nodes":[],"marks":[]
                  },
                  {
                    "id":11,"type":"con","name":"nvim","app_id":null,"pid":2,"shell":"xwayland",
                    "border":"none","current_border_width":0,"layout":"none","orientation":"none",
                    "rect":{"x":960,"y":0,"width":960,"height":1080},
                    "window_rect":{"x":960,"y":0,"width":960,"height":1080},
                    "deco_rect":{"x":0,"y":0,"width":0,"height":0},
                    "geometry":{"x":960,"y":0,"width":960,"height":1080},
                    "urgent":false,"focused":false,"focus":[],"sticky":false,
                    "nodes":[],"floating_nodes":[],"marks":[]
                  }
                ]
              },
              {
                "id":4,"type":"workspace","name":"2",
                "border":"none","current_border_width":0,"layout":"splith","orientation":"horizontal",
                "rect":{"x":0,"y":0,"width":1920,"height":1080},
                "window_rect":{"x":0,"y":0,"width":0,"height":0},
                "deco_rect":{"x":0,"y":0,"width":0,"height":0},
                "geometry":{"x":0,"y":0,"width":0,"height":0},
                "urgent":false,"focused":false,"focus":[],"sticky":false,"marks":[],
                "nodes":[
                  {
                    "id":20,"type":"con","name":null,"pid":null,
                    "border":"none","current_border_width":0,"layout":"splitv","orientation":"vertical",
                    "rect":{"x":0,"y":0,"width":960,"height":1080},
                    "window_rect":{"x":0,"y":0,"width":0,"height":0},
                    "deco_rect":{"x":0,"y":0,"width":0,"height":0},
                    "geometry":{"x":0,"y":0,"width":0,"height":0},
                    "urgent":false,"focused":false,"focus":[],"sticky":false,"marks":[],
                    "floating_nodes":[],
                    "nodes":[
                      {
                        "id":21,"type":"con","name":"kitty","app_id":"kitty","pid":3,"shell":"xdg_shell",
                        "border":"none","current_border_width":0,"layout":"none","orientation":"none",
                        "rect":{"x":0,"y":0,"width":960,"height":540},
                        "window_rect":{"x":0,"y":0,"width":960,"height":540},
                        "deco_rect":{"x":0,"y":0,"width":0,"height":0},
                        "geometry":{"x":0,"y":0,"width":960,"height":540},
                        "urgent":false,"focused":false,"focus":[],"sticky":false,
                        "nodes":[],"floating_nodes":[],"marks":[]
                      },
                      {
                        "id":22,"type":"con","name":"alacritty","app_id":"alacritty","pid":4,"shell":"xdg_shell",
                        "border":"none","current_border_width":0,"layout":"none","orientation":"none",
                        "rect":{"x":0,"y":540,"width":960,"height":540},
                        "window_rect":{"x":0,"y":540,"width":960,"height":540},
                        "deco_rect":{"x":0,"y":0,"width":0,"height":0},
                        "geometry":{"x":0,"y":540,"width":960,"height":540},
                        "urgent":false,"focused":false,"focus":[],"sticky":false,
                        "nodes":[],"floating_nodes":[],"marks":[]
                      }
                    ]
                  },
                  {
                    "id":23,"type":"con","name":"Mozilla Firefox","app_id":"firefox","pid":5,"shell":"xdg_shell",
                    "border":"none","current_border_width":0,"layout":"none","orientation":"none",
                    "rect":{"x":960,"y":0,"width":960,"height":1080},
                    "window_rect":{"x":960,"y":0,"width":960,"height":1080},
                    "deco_rect":{"x":0,"y":0,"width":0,"height":0},
                    "geometry":{"x":960,"y":0,"width":960,"height":1080},
                    "urgent":false,"focused":false,"focus":[],"sticky":false,
                    "nodes":[],"floating_nodes":[],"marks":[]
                  }
                ],
                "floating_nodes":[
                  {
                    "id":24,"type":"floating_con","name":"pavucontrol","app_id":"pavucontrol","pid":6,"shell":"xdg_shell",
                    "border":"none","current_border_width":0,"layout":"none","orientation":"none",
                    "rect":{"x":300,"y":300,"width":600,"height":400},
                    "window_rect":{"x":300,"y":300,"width":600,"height":400},
                    "deco_rect":{"x":0,"y":0,"width":0,"height":0},
                    "geometry":{"x":300,"y":300,"width":600,"height":400},
                    "urgent":false,"focused":false,"focus":[],"sticky":false,
                    "nodes":[],"floating_nodes":[],"marks":[]
                  }
                ]
              },
              {
                "id":5,"type":"workspace","name":"3",
                "border":"none","current_border_width":0,"layout":"none","orientation":"none",
                "rect":{"x":0,"y":0,"width":1920,"height":1080},
                "window_rect":{"x":0,"y":0,"width":0,"height":0},
                "deco_rect":{"x":0,"y":0,"width":0,"height":0},
                "geometry":{"x":0,"y":0,"width":0,"height":0},
                "urgent":false,"focused":false,"focus":[],"sticky":false,"marks":[],
                "nodes":[],"floating_nodes":[]
              }
            ]
          }]
        }"#;

        let root: swayipc::Node = serde_json::from_str(json).unwrap();
        let result = get_ws_and_apps_json(&root);

        // ws "1": two direct children — app_id wins, xwayland falls back to name
        let mut ws1 = result["1"].clone();
        ws1.sort();
        assert_eq!(ws1, vec!["foot", "nvim"]);

        // ws "2": recurses into intermediate split-con (no pid), collects its children,
        //         plus a direct sibling con and a floating_con at workspace level
        let mut ws2 = result["2"].clone();
        ws2.sort();
        assert_eq!(ws2, vec!["alacritty", "firefox", "kitty", "pavucontrol"]);

        // ws "3": empty workspace must be absent from the map
        assert!(!result.contains_key("3"));

        assert_eq!(result.len(), 2);
    }
}
