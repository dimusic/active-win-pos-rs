# active-win-pos-rs

![Build status](https://github.com/dimusic/active-win-pos-rs/actions/workflows/build.yml/badge.svg)

A small Rust library that lets you get position and size of the active window on Windows, MacOS and Linux

## Usage

### Add to Cargo.toml:
```toml
[dependencies]
active-win-pos-rs = "0.5"
```

### Use:
```rust
use active_win_pos_rs::get_active_window;

fn main() {
    match get_active_window() {
        Ok(active_window) => {
            println!("active window: {:#?}", active_window);
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
active window: ActiveWindow {
    window_id: "0x70af2",
    process_id: 22908,
    position: WindowPosition {
        x: 1414.0,
        y: 135.0,
        width: 993.0,
        height: 519.0,
    },
}
```
