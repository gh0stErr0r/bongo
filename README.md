# Bongo Cat Input Overlay

A lightweight desktop widget that displays a Bongo Cat tapping its paws in real-time as you type.

## Demonstration
Tested on **niri** compositor.

https://github.com/user-attachments/assets/f15b4fd8-ef80-43ce-944c-d3116e5d38bc


## Installation

### Prerequisites
Before building, ensure you have the required system libraries installed (GTK4 and Layer Shell). 
For Arch Linux:
```bash
sudo pacman -S gtk4 gtk4-layer-shell
```

### Building from source
Clone the repository and run:
```bash
cargo build --release
sudo cp ./target/release/bongo /usr/bin
```

> **Note:** Since the app reads directly from `/dev/input/`, you might need to run it with `sudo` or add your user to the `input` group to give it permissions to capture keystrokes: `sudo usermod -aG input $USER` (relogin required).

## Configuration
Create a configuration file at `~/.config/bongo.toml`:

```toml
[imgs]
rest = "~/.config/bongo/cat-rest.png"
left = "~/.config/bongo/cat-left.png"
right = "~/.config/bongo/cat-right.png"

[input]
device = "AT Translated Set 2 keyboard"
```
