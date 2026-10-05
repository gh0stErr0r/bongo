use gtk4::gdk;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Image, Orientation};
use serde::Deserialize;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(target_os = "macos")]
use macos as platform;

pub(crate) const STATE_REST: u8 = 0;
pub(crate) const STATE_LEFT: u8 = 1;
pub(crate) const STATE_RIGHT: u8 = 2;

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
    pub window: Option<Window>,
}

#[derive(Debug, Deserialize, Clone)]
struct Window {
    pub position: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            imgs: None,
            input: None,
            window: None,
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

fn image_path(configured_path: Option<String>, fallback_name: &str) -> String {
    if let Some(path) = expand_path(configured_path) {
        if Path::new(&path).exists() {
            return path;
        }
        eprintln!("[WARN] Image not found: {path}");
    }

    let bundled_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("imgs")
        .join(fallback_name);
    bundled_path.to_string_lossy().into_owned()
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

    if let Err(error) = gtk4::init() {
        eprintln!("Failed to initialize GTK: {error}");
        return glib::ExitCode::new(1);
    }

    let app = Application::new(
        Some("com.bongocat.widget"),
        gtk4::gio::ApplicationFlags::empty(),
    );

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

    let position = conf
        .window
        .as_ref()
        .and_then(|window| window.position.clone())
        .unwrap_or_else(|| "top-left".to_string());

    platform::configure_window(&window, &position);

    let container = Box::builder()
        .orientation(Orientation::Horizontal)
        .css_name("bongo-container")
        .build();

    let img_rest_path = image_path(
        conf.imgs.as_ref().and_then(|i| i.rest.clone()),
        "cat-rest.png",
    );
    let img_left_path = image_path(
        conf.imgs.as_ref().and_then(|i| i.left.clone()),
        "cat-left.png",
    );
    let img_right_path = image_path(
        conf.imgs.as_ref().and_then(|i| i.right.clone()),
        "cat-right.png",
    );

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

    platform::start_input(state_input, target_device_name);
}
