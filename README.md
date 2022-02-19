# active-win-pos-rs

A small Rust library that let's you get position and size of the active window on Windows and MacOS

## Usage

Add to Cargo.toml:
```toml
[dependencies]
active-win-pos-rs = "0.2.0"
```

Use:
```rust
use active_win_pos_rs::get_position;

fn main() {
    match get_position() {
        Ok(window_position) => {
            println!("window position: {:?}", window_position);
        },
        Err(()) => {
            println!("error occurred while getting window position");
        }
    }
}
```

## Build

```sh
% git clone https://github.com/dimusic/active-win-pos-rs.git
% cd active-win-pos-rs
% cargo build
```

## Example
```sh
% cargo run --example simple
```
Output:
```
% window position: WindowPosition { x: 733.0, y: 353.0, width: 815.0, height: 583.0 }
```