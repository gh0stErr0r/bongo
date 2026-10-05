use gtk4::gdk;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Image, Orientation};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use serde::Deserialize;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

const DEFAULT_IMG_REST: &str = "~/.config/bongo/cat-rest.png";
const DEFAULT_IMG_LEFT: &str = "~/.config/bongo/cat-left.png";
const DEFAULT_IMG_RIGHT: &str = "~/.config/bongo/cat-right.png";

const STATE_REST: u8 = 0;
const STATE_LEFT: u8 = 1;
const STATE_RIGHT: u8 = 2;

#[derive(Debug, Deserialize, Clone)]
struct Imgs {
    pub rest: Option<String>,
    pub left: Option<String>,
    pub right: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct Input {
    pub device: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct Config {
    pub imgs: Option<Imgs>,
    pub input: Option<Input>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            imgs: None,
            input: None,
        }
    }
}

fn expand_path(path_option: Option<String>) -> Option<String> {
    let path = path_option?;
    if path.starts_with('~') {
        if let Ok(home) = std::env::var("HOME") {
            return Some(path.replacen('~', &home, 1));
        }
    }
    Some(path)
}

pub fn main() -> glib::ExitCode {
    let file_path_op = expand_path(Some(String::from("~/.config/bongo.toml")));

    let file_path = match file_path_op {
        None => {
            println!("Something went wrong with path expansion");
            return glib::ExitCode::new(1);
        }
        Some(a) => a,
    };

    let mut data = vec![];
    if let Ok(mut file) = std::fs::File::open(file_path) {
        let _ = file.read_to_end(&mut data);
    }

    let conf: Config = if !data.is_empty() {
        toml::from_str(&String::from_utf8(data).expect("Our bytes should be valid utf8"))
            .unwrap_or_default()
    } else {
        Config::default()
    };

    let app = Application::builder()
        .application_id("com.bongocat.widget")
        .build();

    app.connect_activate(move |app| {
        build_ui(app, conf.clone());
    });

    app.run()
}

fn build_ui(app: &Application, conf: Config) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Bongo Cat Widget")
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Top);

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, false);
    window.set_anchor(Edge::Bottom, false);

    window.set_margin(Edge::Top, 16);
    window.set_margin(Edge::Left, 16);
    window.set_exclusive_zone(0);

    let container = Box::builder()
        .orientation(Orientation::Horizontal)
        .css_name("bongo-container")
        .build();

    let img_rest_path = expand_path(
        conf.imgs
            .as_ref()
            .and_then(|i| i.rest.clone())
            .or_else(|| Some(DEFAULT_IMG_REST.to_string())),
    )
    .unwrap_or_default();

    let img_left_path = expand_path(
        conf.imgs
            .as_ref()
            .and_then(|i| i.left.clone())
            .or_else(|| Some(DEFAULT_IMG_LEFT.to_string())),
    )
    .unwrap_or_default();

    let img_right_path = expand_path(
        conf.imgs
            .as_ref()
            .and_then(|i| i.right.clone())
            .or_else(|| Some(DEFAULT_IMG_RIGHT.to_string())),
    )
    .unwrap_or_default();

    let image = Image::from_file(&img_rest_path);
    image.set_pixel_size(48);

    container.append(&image);
    window.set_child(Some(&container));

    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        "
        window { background-color: transparent; }
        .bongo-container {
            background-color: rgba(30, 30, 46, 0.85);
            border: 2px solid #b4befe;
            border-radius: 12px;
            padding: 8px;
        }
    ",
    );

    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    window.present();

    let current_state = Arc::new(AtomicU8::new(STATE_REST));

    let image_clone = image.clone();
    let state_ui = current_state.clone();
    let mut last_rendered_state = STATE_REST;

    let path_left = img_left_path.clone();
    let path_right = img_right_path.clone();
    let path_rest = img_rest_path.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
        let state = state_ui.load(Ordering::Relaxed);
        if state != last_rendered_state {
            match state {
                STATE_LEFT => image_clone.set_from_file(Some(Path::new(&path_left))),
                STATE_RIGHT => image_clone.set_from_file(Some(Path::new(&path_right))),
                _ => image_clone.set_from_file(Some(Path::new(&path_rest))),
            }
            last_rendered_state = state;
        }
        glib::ControlFlow::Continue
    });

    let state_input = current_state.clone();

    let target_device_name = conf
        .input
        .as_ref()
        .and_then(|i| i.device.clone())
        .unwrap_or_else(|| "AT Translated Set 2 keyboard".to_string());

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
                    if path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .starts_with("event")
                    {
                        match evdev::Device::open(&path) {
                            Ok(device) => {
                                if let Some(name) = device.name() {
                                    println!(
                                        "[INFO] Opened file {:?}, device name: \"{}\"",
                                        path, name
                                    );
                                    if name == target_device_name {
                                        println!(
                                            "[INFO] Match found! Selecting device: {:?}",
                                            path
                                        );
                                        target_device = Some(device);
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                println!("[ERROR] Failed to open file {:?}. Error: {}", path, e);
                                return;
                            }
                        }
                    }
                }
            }

            if let Some(device) = target_device {
                println!("[INFO] Creating event stream...");
                if let Ok(mut event_stream) = device.into_event_stream() {
                    println!("[INFO] Event stream started successfully. Start typing!");
                    let mut strike_left = true;

                    while let Ok(event) = event_stream.next_event().await {
                        if let evdev::EventSummary::Key(_, _, value) = event.destructure() {
                            if value > 0 {
                                if strike_left {
                                    state_input.store(STATE_LEFT, Ordering::Relaxed);
                                } else {
                                    state_input.store(STATE_RIGHT, Ordering::Relaxed);
                                }
                                strike_left = !strike_left;
                            } else {
                                state_input.store(STATE_REST, Ordering::Relaxed);
                            }
                        }
                    }
                } else {
                    println!("[ERROR] Failed to create event stream!");
                }
            }
        });
    });
}
