use crate::{STATE_LEFT, STATE_REST, STATE_RIGHT};
use gtk4::ApplicationWindow;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

pub fn configure_window(window: &ApplicationWindow, position: &str) {
    window.init_layer_shell();
    window.set_layer(Layer::Top);
    let (top, right, bottom, left) = match position {
        "top-right" => (true, true, false, false),
        "bottom-left" => (false, false, true, true),
        "bottom-right" => (false, true, true, false),
        _ => (true, false, false, true),
    };
    window.set_anchor(Edge::Top, top);
    window.set_anchor(Edge::Right, right);
    window.set_anchor(Edge::Bottom, bottom);
    window.set_anchor(Edge::Left, left);
    window.set_margin(Edge::Top, if top { 16 } else { 0 });
    window.set_margin(Edge::Right, if right { 16 } else { 0 });
    window.set_margin(Edge::Bottom, if bottom { 16 } else { 0 });
    window.set_margin(Edge::Left, if left { 16 } else { 0 });
    window.set_exclusive_zone(0);
}

pub fn start_input(state: Arc<AtomicU8>, target_device_name: String) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let mut target_device = None;
            println!("[INFO] Starting scan of /dev/input/...");

            if let Ok(entries) = std::fs::read_dir("/dev/input/") {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path
                        .file_name()
                        .is_some_and(|name| name.to_string_lossy().starts_with("event"))
                    {
                        continue;
                    }

                    match evdev::Device::open(&path) {
                        Ok(device) => {
                            if device.name().is_some_and(|name| name == target_device_name) {
                                println!("[INFO] Match found! Selecting device: {:?}", path);
                                target_device = Some(device);
                                break;
                            }
                        }
                        Err(error) => println!("[ERROR] Failed to open {:?}: {}", path, error),
                    }
                }
            }

            let Some(device) = target_device else {
                println!("[ERROR] Input device was not found");
                return;
            };

            let Ok(mut event_stream) = device.into_event_stream() else {
                println!("[ERROR] Failed to create event stream!");
                return;
            };

            let mut strike_left = true;
            while let Ok(event) = event_stream.next_event().await {
                if let evdev::EventSummary::Key(_, _, value) = event.destructure() {
                    if value > 0 {
                        state.store(
                            if strike_left { STATE_LEFT } else { STATE_RIGHT },
                            Ordering::Relaxed,
                        );
                        strike_left = !strike_left;
                    } else {
                        state.store(STATE_REST, Ordering::Relaxed);
                    }
                }
            }
        });
    });
}
