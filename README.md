# Bongo Cat Input Overlay

A lightweight desktop widget that displays a Bongo Cat tapping its paws in real-time as you type.

## Demonstration
Tested on **niri** compositor.

https://github.com/user-attachments/assets/f15b4fd8-ef80-43ce-944c-d3116e5d38bc


## Installation

### Prerequisites
Before building, install GTK4. Linux also requires GTK4 Layer Shell.

For Arch Linux:
```bash
sudo pacman -S gtk4 gtk4-layer-shell
```

macOS support is not available at this stage. It is planned for a future
version.

### Building from source
Clone the repository and run:
```bash
cd /path/to/bongo
cargo build --release
```

On Linux, the binary can be installed with `sudo cp ./target/release/bongo /usr/bin`.

> **Linux note:** Since the app reads directly from `/dev/input/`, you might need to run it with `sudo` or add your user to the `input` group: `sudo usermod -aG input $USER` (relogin required).

## Configuration
Create a configuration file at `~/.config/bongo.toml`:

```toml
[imgs]
rest = "~/.config/bongo/cat-rest.png"
left = "~/.config/bongo/cat-left.png"
right = "~/.config/bongo/cat-right.png"

[input]
device = "AT Translated Set 2 keyboard"

[window]
# top-left, top-right, bottom-left or bottom-right
position = "top-left"
```
