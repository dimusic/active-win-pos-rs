# active-win-pos-rs

![Build status](https://github.com/dimusic/active-win-pos-rs/actions/workflows/build.yml/badge.svg)

A small Rust library that lets you get position, size, title and a few other properties of the active window on Windows, MacOS and Linux

## Usage

### Add to Cargo.toml:
```toml
[dependencies]
active-win-pos-rs = "0.9"
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
Would give you an instance of ```ActiveWindow``` struct with unique window id, process id, window position and window title.

Or use ``` active_win_pos_rs::get_position ``` to get the ```WindowPosition``` only.

### Window title on MacOS
On MacOS ```title``` property will always return an empty string
unless you [Enable Screen Recording permission](https://support.apple.com/en-ca/guide/mac-help/mchld6aa7d23/mac) for your app.

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
    title: "cmd - cargo  run --example active-window",
    process_path: "C:\\Program Files\\WindowsApps\\Microsoft.WindowsTerminal_1.16.10262.0_x64__8wekyb3d8bbwe\\WindowsTerminal.exe",
    app_name: "WindowsTerminal",
    window_id: "HWND(9700584)",
    process_id: 8460,
    position: WindowPosition {
        x: 6.0,
        y: 296.0,
        width: 1129.0,
        height: 635.0,
    },
}
```
