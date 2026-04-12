#### what is the best tech stack to expose a cli to a server that can control every single aspect of the pc natively (including third-party apps, browsers, etc) to then expose it to a llm to control it?

  This is a very difficult task, you will not be able to answer it without using web-search

● Web Search("LLM computer use agent tech stack 2025 2026 CLI control desktop automation browser native apps")
  ⎿  Did 1 search in 12s

● Web Search("best framework expose computer control LLM MCP tool use desktop automation 2025")
  ⎿  Did 1 search in 8s

● Web Search("Linux desktop automation LLM atspi xdotool at-spi accessibility tree computer use agent 2025")
  ⎿  Did 1 search in 10s

● Web Search("computer use agent architecture screenshot OCR accessibility tree pyautogui xdg tools Linux 2025")
  ⎿  Did 1 search in 8s

● Web Search("Wayland Sway compositor desktop automation LLM agent wlroots wtype ydotool 2025")
  ⎿  Did 1 search in 7s

● Web Search("MCP server expose system tools LLM CLI bash filesystem process control architecture 2025")
  ⎿  Did 1 search in 8s

● This is a genuinely hard problem with many moving parts. Since you're on Fedora + Sway (Wayland), the answer is more constrained than on X11 or Windows. Here's the full
  architecture:

  ---
  The Core Problem

  You need to bridge three distinct worlds:
  1. System/native apps — controlled via input simulation + accessibility APIs
  2. Browsers — controlled via a debug protocol
  3. LLM — needs a structured, typed interface to call all of the above

  ---
  Layer 1: Input Simulation (Wayland-specific)

  Wayland's security model deliberately breaks X11 tools like xdotool. On Sway/wlroots you use:

  ┌───────────────────────────┬───────────────────────────────────────────────────────────┐
  │           Tool            │                          Purpose                          │
  ├───────────────────────────┼───────────────────────────────────────────────────────────┤
  │ ydotool + ydotoold daemon │ Global mouse/keyboard injection (needs uinput perms)      │
  ├───────────────────────────┼───────────────────────────────────────────────────────────┤
  │ wtype                     │ Keyboard-only, no root, works in nested compositors       │
  ├───────────────────────────┼───────────────────────────────────────────────────────────┤
  │ grim + slurp              │ Screenshots (wlroots-native, replaces scrot)              │
  ├───────────────────────────┼───────────────────────────────────────────────────────────┤
  │ wlrctl                    │ Window management (focus, move, resize) via wlr-protocols │
  └───────────────────────────┴───────────────────────────────────────────────────────────┘

  ydotool is the closest thing to xdotool for Wayland — it injects at the kernel uinput level, bypassing Wayland's input isolation entirely.

  ---
  Layer 2: Structured UI Access (No Vision Required)

  Screenshot+OCR is fragile and slow. The right approach is AT-SPI2 — the Linux accessibility tree:

  - Every GTK/Qt/Electron app exposes its widget tree over D-Bus via AT-SPI2
  - You get button labels, roles, states, coordinates — without any vision model
  - Python: pyatspi2 or the newer accerciser for exploration

  Use AT-SPI2 as primary, fall back to screenshot+OCR (via Tesseract or a VLM) only for apps that don't expose an accessibility tree (games, some Electron apps with it
  disabled).

  ---
  Layer 3: Browser Automation

  Use Playwright — not Selenium, not raw CDP. Reasons:
  - Native support for Chromium, Firefox, WebKit
  - Proper async Python/Node API
  - Has a first-class MCP server (@playwright/mcp) already built
  - Handles JS-heavy SPAs, file uploads, shadow DOM, auth state

  For monitoring network traffic or intercepting requests (useful for seeing what a browser app is doing), pair with mitmproxy.

  ---
  Layer 4: System-Level Control

  ┌────────────────────────────────────────────────────────┬────────────────────────────────────────────────────┐
  │                          Need                          │                        Tool                        │
  ├────────────────────────────────────────────────────────┼────────────────────────────────────────────────────┤
  │ Run shell commands / spawn processes                   │ Python subprocess / asyncio.create_subprocess_exec │
  ├────────────────────────────────────────────────────────┼────────────────────────────────────────────────────┤
  │ Manage systemd units                                   │ systemd D-Bus API via pystemd                      │
  ├────────────────────────────────────────────────────────┼────────────────────────────────────────────────────┤
  │ File system ops                                        │ Direct Python pathlib / aiofiles                   │
  ├────────────────────────────────────────────────────────┼────────────────────────────────────────────────────┤
  │ D-Bus app control (media players, notifications, etc.) │ dbus-python or dasbus                              │
  ├────────────────────────────────────────────────────────┼────────────────────────────────────────────────────┤
  │ Clipboard                                              │ wl-clipboard (wl-copy, wl-paste)                   │
  ├────────────────────────────────────────────────────────┼────────────────────────────────────────────────────┤
  │ Volume/audio                                           │ PipeWire via wpctl CLI or pulsectl Python lib      │
  └────────────────────────────────────────────────────────┴────────────────────────────────────────────────────┘

  ---
  Layer 5: MCP — The Glue to the LLM

  Model Context Protocol (MCP) is the right answer for exposing all of the above to any LLM. It's now universally adopted (Anthropic, OpenAI, Google all support it).

  Architecture:

  LLM (Claude/GPT/local)
      ↕ MCP client (stdio or HTTP/SSE transport)
  MCP Server (your Python process)
      ├── tool: screenshot()       → grim
      ├── tool: click(x, y)        → ydotool
      ├── tool: type(text)         → wtype
      ├── tool: get_ui_tree()      → AT-SPI2
      ├── tool: focus_window(name) → wlrctl / swaymsg
      ├── tool: run_shell(cmd)     → subprocess
      ├── tool: browser_navigate() → Playwright
      ├── tool: browser_click()    → Playwright
      └── tool: dbus_call()        → dasbus

  Build the MCP server in Python using the official mcp SDK (pip install mcp). Each capability is a @server.tool() decorated async function.

  ---
  Layer 6: LLM Integration

  ┌───────────────────────────────────────────────────┬──────────────────────────────────────────────────────────────────────────┐
  │                      Option                       │                               When to use                                │
  ├───────────────────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────┤
  │ Claude API (claude-opus-4-6 or claude-sonnet-4-6) │ Best tool-use reasoning, native MCP client support                       │
  ├───────────────────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────┤
  │ OpenAI Agents SDK                                 │ If you prefer OpenAI's ecosystem                                         │
  ├───────────────────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────┤
  │ Ollama (local)                                    │ Air-gapped / privacy-sensitive; quality gap for complex multi-step tasks │
  ├───────────────────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────┤
  │ LangGraph / LlamaIndex                            │ If you need multi-agent orchestration or memory                          │
  └───────────────────────────────────────────────────┴──────────────────────────────────────────────────────────────────────────┘

  For a single-user personal control agent, go directly with Claude's API + MCP — no orchestration framework needed, it adds complexity without benefit at this scale.

  ---
  Recommended Full Stack

  ┌─────────────────────────────────────────────┐
  │  LLM: Claude API (claude-sonnet-4-6)         │
  │  Transport: MCP over stdio                   │
  ├─────────────────────────────────────────────┤
  │  MCP Server: Python (mcp SDK)                │
  │  Orchestration: none (direct tool calls)     │
  ├──────────────┬──────────────┬───────────────┤
  │  Input layer │  UI layer    │  Browser      │
  │  ydotool     │  AT-SPI2     │  Playwright   │
  │  wtype       │  pyatspi2    │  @playwright/ │
  │  grim        │              │  mcp server   │
  ├──────────────┴──────────────┴───────────────┤
  │  System layer                                │
  │  subprocess, pystemd, dasbus, wl-clipboard   │
  │  wpctl, swaymsg (IPC socket)                 │
  └─────────────────────────────────────────────┘

  ---
  Key Gotchas for Sway/Wayland

  1. ydotoold must run as a background daemon with uinput access — add your user to the input group and set up a systemd user service
  2. grim needs WAYLAND_DISPLAY set correctly — ensure it's exported in the service environment
  3. swaymsg gives you IPC into Sway directly (window list, focus, layout) — use it over wlrctl when on Sway specifically
  4. AT-SPI2 must be enabled — set GTK_MODULES=gail:atk-bridge or use dbus-launch if apps don't expose their tree
  5. Electron apps (VSCode, Slack, Discord) expose AT-SPI only if launched with --enable-accessibility

  ---
  Notable Prior Art to Study

  - https://github.com/isac322/kwin-mcp — MCP server for KDE/Wayland desktop automation (30 tools, AT-SPI2 backed)
  - https://github.com/xlang-ai/OpenCUA — open research framework for computer-use agents
  - https://github.com/OTAKUWeBer/Wayland-automation — Python lib for wlroots mouse/keyboard (Sway-compatible)
  - https://dev.to/tlrag/-directshell-i-turned-the-accessibility-layer-into-a-universal-app-interface-no-screenshots-no-2457 — accessibility-tree-as-universal-interface
  concept

  ---
  Sources:
  - https://www.blog.brightcoding.dev/2025/07/31/let-ai-drive-windows-inside-windows-mcp-the-tiny-server-that-lets-any-llm-control-your-desktop/
  - https://github.com/isac322/kwin-mcp
  - https://github.com/OTAKUWeBer/Wayland-automation
  - https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/
  - https://mofeed.xyz/posts/linux-agentic-desktop/
  - https://fedoramagazine.org/automation-through-accessibility/
  - https://github.com/xlang-ai/OpenCUA
  - https://dev.to/tlrag/-directshell-i-turned-the-accessibility-layer-into-a-universal-app-interface-no-screenshots-no-2457
  - https://modelcontextprotocol.io/introduction
  - https://github.com/mark3labs/mcphost
  - https://www.netguru.com/blog/ai-agent-tech-stack
  - https://thenewstack.io/5-key-trends-shaping-agentic-development-in-2026/

✻ Cooked for 1m 37s

❯ Can this be done in rust for inbuilt safety, unit-testing and cross-platform support?

  and like i said earlier I would prefer a cli that will interact with a server. MCPs are dead and bloated.

● Web Search("Rust desktop automation accessibility AT-SPI atspi crate 2025 Linux Wayland")
  ⎿  Did 1 search in 5s

● Web Search("Rust browser automation CDP chromium playwright alternative 2025 headless")
  ⎿  Did 1 search in 8s

● Web Search("Rust input simulation Linux Wayland ydotool uinput enigo crate keyboard mouse 2025")
  ⎿  Did 1 search in 7s

● Web Search("Rust CLI server architecture gRPC JSON-RPC tokio Unix socket IPC LLM tool use 2025")
  ⎿  Did 1 search in 9s

● Web Search("computeruse-rs crate Rust computer use agent 2025")
  ⎿  Did 1 search in 6s

● Web Search("Rust screenshot capture Linux Wayland xcap scap crate 2025")
  ⎿  Did 1 search in 5s

● Yes — and Rust is actually a strong choice here. The ecosystem has matured enough. Here's the full breakdown:

  ---
  Architecture

  LLM (Claude/OpenAI API)
          ↕  HTTP (reqwest)
  ┌───────────────────────────┐
  │   daemon (Rust, tokio)    │  ← the server, runs as a background process
  │                           │
  │  ├─ input    (enigo/uinput)│
  │  ├─ screen   (xcap)        │
  │  ├─ a11y     (atspi)       │
  │  ├─ browser  (chromiumoxide)│
  │  └─ system   (zbus/process)│
  └───────────┬───────────────┘
              ↕  gRPC (tonic)  ← over Unix socket or TCP
  ┌───────────────────────────┐
  │   cli (Rust, clap)        │  ← thin client, what the user types
  └───────────────────────────┘

  The CLI and the LLM talk to the same daemon over the same transport. The LLM can either call the daemon directly over gRPC, or you write a thin LLM bridge that translates
  the LLM's tool-call JSON into gRPC calls.

  ---
  Transport: gRPC (tonic)

  Why gRPC over JSON-RPC or raw sockets:
  - .proto files are the contract — CLI and server can never drift out of sync
  - Auto-generated client/server stubs = no handwritten serialization
  - Streaming RPCs (good for long-running tasks like "watch this window")
  - Mock server in tests is trivial — just implement the trait
  - Language-agnostic if you ever add a Python LLM bridge

  # Cargo.toml
  tonic = { version = "0.12", features = ["transport"] }
  prost = "0.13"
  tokio = { version = "1", features = ["full"] }

  Your .proto schema becomes the source of truth:
  service ControlDaemon {
    rpc Screenshot(ScreenshotRequest) returns (ScreenshotResponse);
    rpc Click(ClickRequest) returns (ActionResult);
    rpc Type(TypeRequest) returns (ActionResult);
    rpc GetUITree(UITreeRequest) returns (UITreeResponse);
    rpc BrowserNavigate(NavigateRequest) returns (ActionResult);
    rpc RunShell(ShellRequest) returns (ShellResponse);
  }

  ---
  Crate-by-Crate Breakdown

  Input Simulation

  ┌──────────────────────┬────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
  │        Crate         │                                                       Notes                                                        │
  ├──────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ enigo                │ Cross-platform (Windows, macOS, Linux X11). Wayland support exists but behind a feature flag (libei), still buggy. │
  ├──────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ mouse-keyboard-input │ uinput-based, Wayland-native, no X11 required. Works on Sway. Needs /dev/uinput permissions.                       │
  └──────────────────────┴────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘

  Recommendation for your Sway setup: use mouse-keyboard-input for Linux, enigo on Windows/macOS, gated with #[cfg(target_os)].

  Screenshot

  xcap — the clear winner. Pure Rust, cross-platform (Linux X11 + Wayland, macOS, Windows). Supports per-monitor and per-window capture.

  xcap = "0.4"

  Accessibility Tree

  ┌──────────┬────────────────────────────────────────────────────────────┐
  │ Platform │                           Crate                            │
  ├──────────┼────────────────────────────────────────────────────────────┤
  │ Linux    │ atspi (odilia-app) — pure Rust, async, zbus-backed AT-SPI2 │
  ├──────────┼────────────────────────────────────────────────────────────┤
  │ Windows  │ windows crate — call UIAutomation COM interfaces directly  │
  ├──────────┼────────────────────────────────────────────────────────────┤
  │ macOS    │ accessibility crate — wraps AXUIElement                    │
  └──────────┴────────────────────────────────────────────────────────────┘

  atspi is the most mature of the three. Windows/macOS AT equivalents are rougher in Rust — you may need raw FFI via the windows or core-foundation crates. Abstract behind a
  trait:

  #[async_trait]
  pub trait AccessibilityBackend {
      async fn get_focused_element(&self) -> Result<UIElement>;
      async fn find_by_label(&self, label: &str) -> Result<Vec<UIElement>>;
      async fn click_element(&self, element: &UIElement) -> Result<()>;
  }

  Browser Automation

  chromiumoxide — async Rust CDP client, drives Chrome/Chromium. Actively maintained, tokio-based.

  chromiumoxide = { version = "0.7", features = ["tokio-runtime"] }

  You connect to a running Chrome instance (--remote-debugging-port=9222) or spawn it yourself. No Node.js, no Python. Works cross-platform wherever Chrome runs.

  System / D-Bus / Process Control

  ┌────────────────┬─────────────────────────────────────────────┐
  │      Need      │                    Crate                    │
  ├────────────────┼─────────────────────────────────────────────┤
  │ D-Bus (Linux)  │ zbus — pure Rust, async, no libdbus dep     │
  ├────────────────┼─────────────────────────────────────────────┤
  │ Shell commands │ tokio::process::Command                     │
  ├────────────────┼─────────────────────────────────────────────┤
  │ File system    │ tokio::fs                                   │
  ├────────────────┼─────────────────────────────────────────────┤
  │ PipeWire/audio │ pipewire crate or shell out to wpctl        │
  ├────────────────┼─────────────────────────────────────────────┤
  │ Sway IPC       │ swayipc crate — typed async Sway IPC client │
  └────────────────┴─────────────────────────────────────────────┘

  ---
  Workspace Layout

  workspace/
  ├── Cargo.toml              # workspace root
  ├── proto/
  │   └── control.proto       # gRPC schema — single source of truth
  ├── crates/
  │   ├── daemon/             # the server binary
  │   │   ├── src/
  │   │   │   ├── main.rs
  │   │   │   ├── input/      # platform-gated input backends
  │   │   │   ├── screen/     # xcap wrapper
  │   │   │   ├── a11y/       # atspi / UIAutomation / AX
  │   │   │   ├── browser/    # chromiumoxide
  │   │   │   └── system/     # zbus, process, swayipc
  │   │   └── build.rs        # tonic prost codegen
  │   ├── cli/                # thin clap client
  │   │   └── src/main.rs     # parses args → gRPC call → prints result
  │   └── llm-bridge/         # optional: reqwest + tool-call → gRPC translation

  ---
  Cross-Platform Gating

  // crates/daemon/src/input/mod.rs
  #[cfg(target_os = "linux")]
  mod linux;
  #[cfg(target_os = "linux")]
  pub use linux::InputBackend;

  #[cfg(target_os = "windows")]
  mod windows;
  #[cfg(target_os = "windows")]
  pub use windows::InputBackend;

  #[cfg(target_os = "macos")]
  mod macos;
  #[cfg(target_os = "macos")]
  pub use macos::InputBackend;

  This keeps platform-specific code quarantined and fully unit-testable per platform in CI.

  ---
  Testing Strategy

  - gRPC contract tests: implement a MockControlDaemon struct, test CLI parsing against it
  - Unit test each backend independently (input, screen, a11y, browser)
  - Integration tests: spin up the daemon in a test harness, drive it via the real gRPC client
  - CI: Linux (AT-SPI, uinput), Windows (UIAutomation), macOS (AX) — each in separate runners since the backends differ

  ---
  Honest Hard Parts

  ┌──────────────────────┬────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
  │       Problem        │                                                            Reality                                                             │
  ├──────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ Wayland input        │ mouse-keyboard-input (uinput) works but needs uinput kernel perms and the ydotoold equivalent daemon pattern                   │
  ├──────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ Cross-platform a11y  │ AT-SPI (Linux) is mature in Rust. Windows UIAutomation in Rust requires raw COM FFI. macOS AX is thin. Budget extra time here. │
  ├──────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ Electron apps        │ Must be launched with --enable-accessibility flag for AT-SPI to expose their tree                                              │
  ├──────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ LLM tool-call format │ You'll write a small translation layer mapping the LLM's JSON tool calls → typed gRPC calls                                    │
  └──────────────────────┴────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘

  ---
  Summary Stack

  CLI:     clap + tonic client
  Server:  tonic + tokio
  Input:   mouse-keyboard-input (Linux) / enigo (others)
  Screen:  xcap
  A11y:    atspi (Linux) / windows-rs UIAutomation / accessibility (macOS)
  Browser: chromiumoxide (CDP)
  System:  zbus + swayipc + tokio::process

  This gives you a single cargo build --release producing two binaries (daemon + cli), no runtime dependencies, full type safety end-to-end, and a clear seam for unit testing
   every layer independently.

  ---
  Sources:
  - https://github.com/enigo-rs/enigo
  - https://crates.io/crates/mouse-keyboard-input
  - https://github.com/nashaofu/xcap
  - https://github.com/odilia-app/atspi
  - https://dev.to/vhub_systems_ed5641f65d59/puppeteer-in-rust-chromiumoxide-and-headlesschrome-vs-the-python-alternative-4ji0
  - https://github.com/hyperium/tonic
  - https://github.com/xayhemLLC/computer-use-runtime
  - https://github.com/sderosiaux/chrome-agent
  - https://browser-use.com/posts/playwright-to-cdp

✻ Cogitated for 1m 45s

❯ and I would like to support multiple OSes, namely, linux with common compositors and macos.

● Web Search("macOS accessibility AXUIElement Rust crate automation 2025 screen reader control")
  ⎿  Did 1 search in 8s

● Web Search("Linux X11 Wayland Hyprland Sway KDE GNOME compositor input screenshot automation differences 2025")
  ⎿  Did 1 search in 9s

● Web Search("Wayland screenshot portal xdg-desktop-portal wlr-screencopy GNOME KDE automation Rust 2025")
  ⎿  Did 1 search in 8s

● Web Search("macOS ScreenCaptureKit Rust screenshot input simulation CGEvent 2025")
  ⎿  Did 1 search in 7s

● Web Search("enigo Wayland libei support status 2025 2026 input simulation")
  ⎿  Did 1 search in 10s

● Web Search("xcap Wayland wlr-screencopy xdg-desktop-portal screenshot Linux support status 2025")
  ⎿  Did 1 search in 9s

● Good — this significantly changes the surface area. The hard part isn't the high-level code, it's the per-compositor, per-OS matrix you have to cover. Let me lay it out
  fully.

  ---
  The Full Compatibility Matrix

  ┌───────────────┬──────────────┬─────────────────────────────────────────┬──────────────────────────┬──────────────────────────┬─────────────────────────────┐
  │  Capability   │     X11      │ Wayland/wlroots (Sway, Hyprland, river) │  Wayland/GNOME (Mutter)  │    Wayland/KDE (KWin)    │            macOS            │
  ├───────────────┼──────────────┼─────────────────────────────────────────┼──────────────────────────┼──────────────────────────┼─────────────────────────────┤
  │ Screenshot    │ xcap         │ xcap (wlr-screencopy)                   │ xdg-portal D-Bus         │ xdg-portal D-Bus         │ screencapturekit-rs         │
  ├───────────────┼──────────────┼─────────────────────────────────────────┼──────────────────────────┼──────────────────────────┼─────────────────────────────┤
  │ Input inject  │ enigo        │ mouse-keyboard-input (uinput)           │ xdg RemoteDesktop portal │ xdg RemoteDesktop portal │ enigo (CGEvent)             │
  ├───────────────┼──────────────┼─────────────────────────────────────────┼──────────────────────────┼──────────────────────────┼─────────────────────────────┤
  │ Accessibility │ atspi        │ atspi                                   │ atspi                    │ atspi                    │ accessibility (AXUIElement) │
  ├───────────────┼──────────────┼─────────────────────────────────────────┼──────────────────────────┼──────────────────────────┼─────────────────────────────┤
  │ Window mgmt   │ xcb/xlib FFI │ compositor IPC (sway/hyprland-rs)       │ D-Bus (Mutter)           │ D-Bus (KWin)             │ AXUIElement / CGWindowList  │
  ├───────────────┼──────────────┼─────────────────────────────────────────┼──────────────────────────┼──────────────────────────┼─────────────────────────────┤
  │ Clipboard     │ arboard      │ wl-clipboard CLI or xdg-portal          │ wl-clipboard / portal    │ wl-clipboard / portal    │ arboard                     │
  └───────────────┴──────────────┴─────────────────────────────────────────┴──────────────────────────┴──────────────────────────┴─────────────────────────────┘

  ---
  Why This Matrix Is Painful

  Screenshot: xcap handles X11 and Wayland wlroots fine. But GNOME and KDE do not implement wlr-screencopy — they only expose org.freedesktop.portal.Screenshot over D-Bus. So
   xcap may prompt the user on GNOME/KDE. The new ext-image-copy-capture-v1 protocol was just merged into Wayland upstream (2025) and aims to unify this, but compositor
  adoption is still catching up.

  Input injection: This is the most fragmented piece.
  - X11 — enigo sends X events directly, no restrictions
  - wlroots (Sway, Hyprland) — no compositor-level injection API. Your only real option is uinput at the kernel level (mouse-keyboard-input crate), which bypasses Wayland
  entirely. Needs udev rule for /dev/uinput.
  - GNOME — uses libei via the xdg RemoteDesktop portal. enigo's libei feature flag covers this but is still buggy as of late 2025.
  - KDE — also supports xdg RemoteDesktop portal (libei)
  - macOS — enigo via CGEvent, works well

  The cleanest unified strategy: use mouse-keyboard-input (uinput) for all Linux — it works at the kernel level, completely bypassing the display server. On macOS use enigo.
  Gate with #[cfg].

  ---
  Compositor IPC (Window Management)

  This is an extra layer on top of input/screenshot — and it's entirely compositor-specific:

  ┌────────────┬─────────────────────────────────┬───────────────────────────────────┐
  │ Compositor │           IPC Method            │            Rust Crate             │
  ├────────────┼─────────────────────────────────┼───────────────────────────────────┤
  │ Sway       │ Unix socket, JSON protocol      │ https://crates.io/crates/swayipc  │
  ├────────────┼─────────────────────────────────┼───────────────────────────────────┤
  │ Hyprland   │ Unix socket, custom protocol    │ https://crates.io/crates/hyprland │
  ├────────────┼─────────────────────────────────┼───────────────────────────────────┤
  │ GNOME      │ D-Bus (Mutter + Shell DBus API) │ zbus                              │
  ├────────────┼─────────────────────────────────┼───────────────────────────────────┤
  │ KDE        │ D-Bus (KWin scripting + DBus)   │ zbus                              │
  ├────────────┼─────────────────────────────────┼───────────────────────────────────┤
  │ X11        │ EWMH/ICCCM via xcb              │ xcb                               │
  ├────────────┼─────────────────────────────────┼───────────────────────────────────┤
  │ macOS      │ AXUIElement + CGWindowServer    │ accessibility crate               │
  └────────────┴─────────────────────────────────┴───────────────────────────────────┘

  For window focus, listing open windows, moving/resizing — you need compositor IPC. There is no unified abstraction crate for this. You write a trait and implement it per
  compositor:

  #[async_trait]
  pub trait WindowManager: Send + Sync {
      async fn list_windows(&self) -> Result<Vec<WindowInfo>>;
      async fn focus(&self, id: WindowId) -> Result<()>;
      async fn move_resize(&self, id: WindowId, rect: Rect) -> Result<()>;
      async fn active_window(&self) -> Result<WindowInfo>;
  }

  // implementations:
  // SwayWindowManager (swayipc)
  // HyprlandWindowManager (hyprland crate)
  // GnomeWindowManager (zbus → org.gnome.Shell)
  // KdeWindowManager (zbus → org.kde.KWin)
  // X11WindowManager (xcb + EWMH)
  // MacosWindowManager (accessibility + CoreGraphics FFI)

  The daemon detects which backend to use at startup — check $WAYLAND_DISPLAY, $DISPLAY, $XDG_CURRENT_DESKTOP, $SWAYSOCK, HYPRLAND_INSTANCE_SIGNATURE.

  ---
  macOS-Specific Stack

  macOS requires permissions for almost everything — you must handle them gracefully at startup:

  ┌────────────────────┬────────────────────────────┬───────────────────────────────────┬────────────────────────┐
  │     Capability     │            API             │            Rust Crate             │       Permission       │
  ├────────────────────┼────────────────────────────┼───────────────────────────────────┼────────────────────────┤
  │ Screenshot         │ ScreenCaptureKit           │ screencapturekit-rs               │ Screen Recording (TCC) │
  ├────────────────────┼────────────────────────────┼───────────────────────────────────┼────────────────────────┤
  │ Input inject       │ CGEvent                    │ enigo                             │ Accessibility (TCC)    │
  ├────────────────────┼────────────────────────────┼───────────────────────────────────┼────────────────────────┤
  │ Accessibility tree │ AXUIElement                │ accessibility (eiz/accessibility) │ Accessibility (TCC)    │
  ├────────────────────┼────────────────────────────┼───────────────────────────────────┼────────────────────────┤
  │ Window list        │ CGWindowListCopyWindowInfo │ core-graphics (via objc2)         │ none for basic list    │
  ├────────────────────┼────────────────────────────┼───────────────────────────────────┼────────────────────────┤
  │ Clipboard          │ NSPasteboard               │ arboard                           │ none                   │
  └────────────────────┴────────────────────────────┴───────────────────────────────────┴────────────────────────┘

  axcli (mentioned in results) shows the pattern for driving macOS apps via AX — it's a CLI built on the same accessibility crate you'd use. Treat it as a reference
  implementation.

  macOS permission check at startup:
  #[cfg(target_os = "macos")]
  fn check_permissions() -> Result<()> {
      // AXIsProcessTrusted() via accessibility-sys
      // CGRequestScreenCaptureAccess() via core-graphics
      // fail fast with clear error messages directing user to System Settings
  }

  ---
  Updated Crate Map

  [dependencies]
  # Async runtime
  tokio = { version = "1", features = ["full"] }

  # IPC transport
  tonic = "0.12"
  prost = "0.13"

  # CLI
  clap = { version = "4", features = ["derive"] }

  # Screenshot (cross-platform)
  xcap = "0.4"                    # X11 + wlroots Wayland + macOS + Windows

  # Input (Linux - kernel level, all compositors)
  [target.'cfg(target_os = "linux")'.dependencies]
  mouse-keyboard-input = "0.9"   # uinput, works X11 + all Wayland compositors
  atspi = "0.23"                 # AT-SPI2 accessibility tree
  zbus = "5"                     # D-Bus for GNOME/KDE IPC + system control
  swayipc = "3"                  # Sway IPC (optional, feature-gated)
  hyprland = "0.4"               # Hyprland IPC (optional, feature-gated)

  # Input (macOS)
  [target.'cfg(target_os = "macos")'.dependencies]
  enigo = "0.2"                  # CGEvent-based, works well on macOS
  screencapturekit = "0.3"       # ScreenCaptureKit for screenshots
  accessibility = "0.1"          # AXUIElement bindings

  # Shared
  arboard = "3"                  # Clipboard, cross-platform
  chromiumoxide = { version = "0.7", features = ["tokio-runtime"] }  # Browser via CDP
  reqwest = { version = "0.12", features = ["json"] }  # HTTP for LLM API calls

  ---
  Feature Flags for Compositor Backends

  Avoid pulling in all compositor crates on every build:

  [features]
  default = ["auto-detect"]
  auto-detect = []
  sway = ["dep:swayipc"]
  hyprland = ["dep:hyprland"]
  gnome = []    # pure zbus, no extra dep
  kde = []      # pure zbus, no extra dep
  x11 = ["dep:xcb"]

  At startup, auto-detect probes environment variables and activates the right backend dynamically, falling back gracefully:

  fn detect_compositor() -> CompositorKind {
      if env::var("SWAYSOCK").is_ok() { return CompositorKind::Sway; }
      if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() { return CompositorKind::Hyprland; }
      match env::var("XDG_CURRENT_DESKTOP").as_deref() {
          Ok("GNOME") => CompositorKind::Gnome,
          Ok("KDE")   => CompositorKind::Kde,
          _           => {
              if env::var("WAYLAND_DISPLAY").is_ok() { CompositorKind::WaylandGeneric }
              else { CompositorKind::X11 }
          }
      }
  }

  ---
  What You'll Actually Struggle With

  ┌─────────────────────────────┬─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
  │         Pain Point          │                                                                 Reality                                                                 │
  ├─────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ GNOME screenshot without    │ xdg-portal shows a permission dialog every time unless you use a trusted portal client. May need pipewire streaming path instead.       │
  │ prompt                      │                                                                                                                                         │
  ├─────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ Input on GNOME Wayland      │ libei via RemoteDesktop portal is the only clean path. enigo's libei support is experimental. Short-term: uinput still works on GNOME   │
  │                             │ if user has uinput perms.                                                                                                               │
  ├─────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ macOS TCC permissions       │ Can't programmatically grant — user must go to System Settings. Build a clear first-run setup wizard.                                   │
  ├─────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ Hyprland ABI stability      │ The hyprland Rust crate tracks Hyprland closely but IPC protocol can change across Hyprland versions.                                   │
  ├─────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ Browser on Linux without X  │ Chrome with --headless=new works on Wayland. But non-headless Chrome still needs --ozone-platform=wayland flag to run correctly.        │
  └─────────────────────────────┴─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘

  ---
  Final Architecture (Updated)

  daemon/
  ├── transport/      gRPC (tonic) over Unix socket
  ├── screen/
  │   ├── xcap        ← X11 + wlroots + macOS
  │   └── portal      ← GNOME/KDE (org.freedesktop.portal.Screenshot via zbus)
  ├── input/
  │   ├── uinput      ← all Linux (mouse-keyboard-input)
  │   └── cgevent     ← macOS (enigo)
  ├── a11y/
  │   ├── atspi       ← Linux (all DEs, GTK/Qt apps)
  │   └── axui        ← macOS (AXUIElement)
  ├── wm/
  │   ├── sway        ← swayipc
  │   ├── hyprland    ← hyprland crate
  │   ├── gnome/kde   ← zbus D-Bus
  │   ├── x11         ← xcb + EWMH
  │   └── macos       ← accessibility + CGWindowServer
  ├── browser/        chromiumoxide (CDP, cross-platform)
  └── system/
      ├── linux       zbus (systemd, pipewire, dbus services)
      └── macos       NSWorkspace + launchctl via Command

  ---
  Sources:
  - https://github.com/nashaofu/xcap
  - https://github.com/svtlabs/screencapturekit-rs
  - https://github.com/enigo-rs/enigo
  - https://crates.io/crates/mouse-keyboard-input
  - https://github.com/odilia-app/atspi
  - https://github.com/eiz/accessibility
  - https://github.com/next-slide-please/macos-accessibility-client
  - https://www.phoronix.com/news/libei-1.0-Emulated-Input
  - https://wayland.app/protocols/wlr-screencopy-unstable-v1
  - https://www.phoronix.com/news/Wayland-Merges-Screen-Capture
  - https://wiki.archlinux.org/title/XDG_Desktop_Portal
  - https://crates.io/crates/axcli
