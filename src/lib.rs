#![feature(specialization)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate plygui_api;
#[macro_use]
extern crate plygui_macros;

extern crate webview_sys;

#[cfg(all(target_os = "macos", feature = "cocoa_"))]
#[macro_use]
extern crate objc;

#[cfg(all(target_os = "macos", feature = "cocoa_"))]
extern crate plygui_cocoa;

#[cfg(all(target_os = "windows", feature = "win32"))]
extern crate plygui_win32;

#[cfg(feature = "qt5")]
extern crate plygui_qt;

#[cfg(feature = "gtk3")]
extern crate plygui_gtk;

pub mod sdk;

pub mod imp;
pub mod api;

pub use crate::api::webview::{Webview, NewWebview};