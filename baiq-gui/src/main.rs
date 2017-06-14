#![cfg_attr(feature="lint", feature(plugin))]
#![cfg_attr(feature="lint", plugin(clippy))]

extern crate baimax;
extern crate chrono;
#[macro_use]
extern crate clear_coat;
extern crate iup_sys;
extern crate penny;
extern crate serde;
extern crate serde_json;

use clear_coat as cc;

fn main() {
    let dialog = cc::Dialog::new();

    dialog.set_title("baiq");

    dialog
        .show_xy(
            cc::ScreenPosition::CenterParent,
            cc::ScreenPosition::CenterParent,
        )
        .expect("Failed to show the window");
    cc::main_loop();
}
