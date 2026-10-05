use crate::{STATE_LEFT, STATE_REST, STATE_RIGHT};
use gtk4::ApplicationWindow;
use gtk4::prelude::*;
use rdev::{Event, EventType, Key, listen};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn CGPreflightListenEventAccess() -> bool;
    fn CGRequestListenEventAccess() -> bool;
}

pub fn configure_window(window: &ApplicationWindow, position: &str) {
    window.set_decorated(false);
    window.set_resizable(false);
    window.set_default_size(80, 80);
    let _ = position;
}

pub fn start_input(state: Arc<AtomicU8>, _target_device_name: String) {
    std::thread::spawn(move || {
        let access_granted = unsafe {
            if CGPreflightListenEventAccess() {
                true
            } else {
                println!("[WARN] macOS keyboard access is not granted; requesting permission...");
                CGRequestListenEventAccess()
            }
        };

        if !access_granted {
            println!(
                "[ERROR] macOS denied keyboard access. Enable Accessibility for the terminal or launcher, then restart Bongo Cat."
            );
            return;
        }

        let mut strike_left = true;
        let mut received_key = false;
        println!("[INFO] Starting macOS keyboard event tap...");
        let callback = move |event: Event| match event.event_type {
            EventType::KeyPress(key) if !is_modifier(key) => {
                if !received_key {
                    println!("[INFO] macOS keyboard event received");
                    received_key = true;
                }
                state.store(
                    if strike_left { STATE_LEFT } else { STATE_RIGHT },
                    Ordering::Relaxed,
                );
                strike_left = !strike_left;
            }
            EventType::KeyRelease(key) if !is_modifier(key) => {
                state.store(STATE_REST, Ordering::Relaxed)
            }
            _ => {}
        };

        if let Err(error) = listen(callback) {
            println!("[ERROR] macOS keyboard listener stopped: {:?}", error);
        }
    });
}

fn is_modifier(key: Key) -> bool {
    matches!(
        key,
        Key::Alt
            | Key::AltGr
            | Key::CapsLock
            | Key::ControlLeft
            | Key::ControlRight
            | Key::MetaLeft
            | Key::MetaRight
            | Key::ShiftLeft
            | Key::ShiftRight
    )
}
