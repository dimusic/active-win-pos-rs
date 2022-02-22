# active-win-pos-rs

![Build status](https://github.com/dimusic/active-win-pos-rs/actions/workflows/build.yml/badge.svg)

A small Rust library that lets you get position and size of the active window on Windows, MacOS and Linux

## Usage

### Add to Cargo.toml:
```toml
[dependencies]
active-win-pos-rs = "0.4"
```

### Use:
```rust
use active_win_pos_rs::get_active_window;

fn main() {
    match get_active_window() {
        Ok(active_window) => {
            println!("active window: {:?}", active_window);
        },
        Err(()) => {
            println!("error occurred while getting the active window");
        }
    }
}
```
Would give you an instance of ```ActiveWindow``` struct with unique window id, process id and window position.

Or use ``` active_win_pos_rs::get_position ``` to get the ```WindowPosition``` only.

## Build

```sh
% git clone https://github.com/dimusic/active-win-pos-rs.git
% cd active-win-pos-rs
% cargo build
```

## Example
```sh
% cargo run --example active-window
```
Output:
```
% active window: ActiveWindow { window_id: "5704", process_id: 80726, position: WindowPosition { x: 798.0, y: 193.0, width: 815.0, height: 606.0 } }
```
