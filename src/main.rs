use std::collections::HashMap;
use std::process;

use wayland_client::{protocol::wl_registry, Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols_wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1::{self, ZwlrForeignToplevelHandleV1},
    zwlr_foreign_toplevel_manager_v1::{self, ZwlrForeignToplevelManagerV1},
};

// State value for "activated" per the wlr-foreign-toplevel-management protocol
const STATE_ACTIVATED: u32 = 2;

#[derive(Debug, Default, Clone)]
struct ToplevelInfo {
    title: String,
    app_id: String,
    activated: bool,
}

struct AppState {
    manager: Option<ZwlrForeignToplevelManagerV1>,
    toplevels: HashMap<u32, ToplevelInfo>,
    protocol_supported: bool,
}

impl AppState {
    fn new() -> Self {
        Self {
            manager: None,
            toplevels: HashMap::new(),
            protocol_supported: false,
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == "zwlr_foreign_toplevel_manager_v1" {
                let mgr: ZwlrForeignToplevelManagerV1 = registry.bind(name, version.min(3), qh, ());
                state.manager = Some(mgr);
                state.protocol_supported = true;
            }
        }
    }
}

impl Dispatch<ZwlrForeignToplevelManagerV1, ()> for AppState {
    fn event(
        state: &mut Self,
        _manager: &ZwlrForeignToplevelManagerV1,
        event: zwlr_foreign_toplevel_manager_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } => {
                let id = toplevel.id().protocol_id();
                state.toplevels.entry(id).or_default();
            }
            zwlr_foreign_toplevel_manager_v1::Event::Finished => {}
            _ => {}
        }
    }

    fn event_created_child(
        _opcode: u16,
        qhandle: &QueueHandle<Self>,
    ) -> std::sync::Arc<dyn wayland_client::backend::ObjectData> {
        qhandle.make_data::<ZwlrForeignToplevelHandleV1, ()>(())
    }
}

impl Dispatch<ZwlrForeignToplevelHandleV1, ()> for AppState {
    fn event(
        state: &mut Self,
        proxy: &ZwlrForeignToplevelHandleV1,
        event: zwlr_foreign_toplevel_handle_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let id = proxy.id().protocol_id();
        let info = state.toplevels.entry(id).or_default();

        match event {
            zwlr_foreign_toplevel_handle_v1::Event::Title { title } => {
                info.title = title;
            }
            zwlr_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
                info.app_id = app_id;
            }
            zwlr_foreign_toplevel_handle_v1::Event::State { state: raw } => {
                info.activated = raw
                    .chunks_exact(4)
                    .map(|chunk| u32::from_ne_bytes(chunk.try_into().unwrap()))
                    .any(|v| v == STATE_ACTIVATED);
            }
            zwlr_foreign_toplevel_handle_v1::Event::Closed => {
                state.toplevels.remove(&id);
            }
            zwlr_foreign_toplevel_handle_v1::Event::Done => {}
            _ => {}
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let json_output = args.iter().any(|a| a == "--json" || a == "-j");
    let show_help = args.iter().any(|a| a == "--help" || a == "-h");

    if show_help {
        eprintln!("Usage: active-window [OPTIONS]");
        eprintln!();
        eprintln!("Prints the currently active Wayland window to stdout.");
        eprintln!("Requires a compositor supporting zwlr_foreign_toplevel_manager_v1");
        eprintln!("(sway, Hyprland, and most wlroots-based compositors).");
        eprintln!();
        eprintln!("Options:");
        eprintln!("  -j, --json    Output as JSON: {{\"title\":\"...\",\"app_id\":\"...\"}}");
        eprintln!("  -h, --help    Show this help");
        process::exit(0);
    }

    let conn = match Connection::connect_to_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: could not connect to Wayland display: {e}");
            eprintln!("Is WAYLAND_DISPLAY set and a compositor running?");
            process::exit(1);
        }
    };

    let mut event_queue = conn.new_event_queue::<AppState>();
    let qh = event_queue.handle();

    conn.display().get_registry(&qh, ());

    let mut state = AppState::new();

    // Roundtrip 1: discover globals, bind manager
    if let Err(e) = event_queue.roundtrip(&mut state) {
        eprintln!("error: Wayland roundtrip failed: {e}");
        process::exit(1);
    }

    if !state.protocol_supported {
        eprintln!("error: compositor does not support zwlr_foreign_toplevel_manager_v1");
        eprintln!("Supported compositors: sway, Hyprland, and other wlroots-based compositors.");
        eprintln!("Note: GNOME/Mutter does not expose this protocol by default.");
        process::exit(1);
    }

    // Roundtrip 2: receive toplevel handles
    if let Err(e) = event_queue.roundtrip(&mut state) {
        eprintln!("error: Wayland roundtrip failed: {e}");
        process::exit(1);
    }

    // Roundtrip 3: receive title/app_id/state/done for each handle
    if let Err(e) = event_queue.roundtrip(&mut state) {
        eprintln!("error: Wayland roundtrip failed: {e}");
        process::exit(1);
    }

    match state.toplevels.values().find(|t| t.activated) {
        Some(info) => {
            if json_output {
                let title = json_escape(&info.title);
                let app_id = json_escape(&info.app_id);
                println!("{{\"title\":\"{title}\",\"app_id\":\"{app_id}\"}}");
            } else {
                println!("{}", info.title);
            }
        }
        None => {
            if state.toplevels.is_empty() {
                eprintln!("no windows reported by compositor");
            } else {
                eprintln!(
                    "no active window found ({} windows visible)",
                    state.toplevels.len()
                );
            }
            process::exit(1);
        }
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}
