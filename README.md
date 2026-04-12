# dunnoPC

A CLI tool that gives an LLM full control over a desktop. Every command is documented in a markdown doc so LLM can call them directly as shell tools to complete tasks autonomously.

## What it does

LLM reads the screen state (as structured data, not pixels), finds what it needs, and acts — without expensive screenshot-driven coordinate guessing for the vast majority of tasks.

## How control works

### Native apps — accessibility tree (primary)

Each platform exposes a native accessibility tree. dunnoPC queries it as JSON and interacts by element name/role — no coordinates.

```
dunnopc ui-tree "firefox"              # dump full widget tree as JSON
dunnopc ui-click "gedit" "Open"        # click button by label
dunnopc ui-type "gedit" "editor" "hi"  # type into named field
dunnopc ui-read "gedit" "editor"       # read text content of element
```

| Platform | Interface | App coverage |
|---|---|---|
| Linux | AT-SPI2 (D-Bus) | GTK, Qt, LibreOffice, Electron\* |
| macOS | AX (Accessibility API) | All native apps, Electron\* |
| Windows | UIAutomation (COM) | Win32, WPF, UWP, Electron\* |

\* Electron apps require `--enable-accessibility` flag at launch.

### Browser — CDP (primary)

Chrome DevTools Protocol gives full DOM access. No screenshots needed for any standard web interaction.

```
dunnopc browser navigate "https://example.com"
dunnopc browser snapshot               # full DOM + a11y tree as JSON
dunnopc browser click "#submit"        # by CSS selector
dunnopc browser click "aria:Search"    # by aria label
dunnopc browser type "input[name=q]" "hello"
dunnopc browser eval "document.title"  # run JS, get result
dunnopc browser wait "#result"         # block until element appears
```

| Browser | Engine | CDP support |
|---|---|---|
| Chrome, Chromium, Brave, Edge, Opera | Chromium | Full |
| Firefox, Zen, Librewolf | Gecko | Partial (Firefox 129+) |
| Safari | WebKit | No CDP — not supported |

All Chromium-based browsers: launch with `--remote-debugging-port=9222 --remote-allow-origins='*'`.
Firefox-based: `--remote-debugging-port=9222` (Firefox 129+), most CDP features work.

Browser holds all session state (tabs, cookies, auth) between calls. No daemon needed.

### Fallback — coordinates + screenshot

Only for apps with no accessibility tree (games, canvas UIs, anything that doesn't expose a11y).

```
dunnopc screenshot                     # saves PNG, returns path
dunnopc click 960 540
dunnopc scroll 960 540 down
```

### Input + window management

```
dunnopc type "hello world"             # types to focused window
dunnopc key "ctrl+c"                   # key combo
dunnopc windows                        # list open windows as JSON
dunnopc focus "firefox"                # focus window by app name
dunnopc shell "cmd"                    # run shell command, stream output
```

## Tech stack

| Layer | Linux | macOS | Windows |
|---|---|---|---|
| CLI | `clap` | `clap` | `clap` |
| A11y tree | `atspi` (AT-SPI2 / zbus) | `accessibility` crate (AX API) | `windows-rs` (UIAutomation) |
| Browser | `chromiumoxide` (CDP) | `chromiumoxide` (CDP) | `chromiumoxide` (CDP) |
| Keyboard | `wtype` (Wayland) / `xdotool` (X11) | CGEventPost | `enigo` / SendInput |
| Mouse | `ydotool` (Wayland) / `xdotool` (X11) | CGEventPost | `enigo` / SendInput |
| Screenshots | `grim` (Wayland) / `scrot` (X11) | `screencapture` | Win32 BitBlt |
| Window mgmt | `swaymsg` / `wmctrl` | AppleScript / AX | Win32 API |
| Shell | `tokio::process` | `tokio::process` | `tokio::process` |
| Runtime | `tokio` | `tokio` | `tokio` |

Platform-specific backends are gated with `#[cfg(target_os)]`. The CLI interface is identical across platforms.

## Workspace layout

```
dunnoPC/
├── src/
│   ├── main.rs          # CLI entry point (clap subcommands)
│   ├── a11y/            # platform-gated accessibility backends
│   ├── browser/         # chromiumoxide CDP client
│   ├── input/           # platform-gated input backends
│   ├── screen/          # platform-gated screenshot backends
│   └── system/          # shell, window management
├── Cargo.toml
└── COMMANDS.md          # documents every command for the LLM
```

## LLM integration

Document every subcommand, its arguments, and its JSON output format. The LLM reads it at session start and uses `dunnopc` to complete tasks involving any app on the desktop.
