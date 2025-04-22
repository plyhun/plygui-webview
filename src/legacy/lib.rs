#[macro_use]
extern crate plygui_api;

#[cfg(all(target_os = "windows", feature = "win32"))]
#[macro_use]
extern crate lazy_static;
#[cfg(all(target_os = "windows", feature = "win32"))]
mod lib_win32;

#[cfg(feature = "gtk3")]
mod lib_gtk;
#[cfg(feature = "gtk3")]
#[macro_use]
extern crate plygui_gtk;
#[cfg(feature = "gtk3")]
extern crate gtk;
#[cfg(feature = "gtk3")]
extern crate webkit2gtk;

#[cfg(all(target_os = "macos", feature = "cocoa_"))]
#[macro_use]
extern crate lazy_static;
#[cfg(all(target_os = "macos", feature = "cocoa_"))]
mod lib_cocoa;
#[cfg(all(target_os = "macos", feature = "cocoa_"))]
#[macro_use]
extern crate objc;

use plygui_api::controls::*;
use plygui_api::sdk::*;

define! {
    WebView: Control {
        constructor: {
            fn with_url<S: AsRef<str>>(url: S) -> Box<dyn WebView>;
        }
        outer: {
            fn set_url(&mut self, site: &str);
            fn url(&self) -> ::std::borrow::Cow<str>;
        }
        inner: {
            fn set_url(&mut self, member: &mut MemberBase, control: &mut ControlBase, site: &str);
            fn url(&self, member: &MemberBase, control: &ControlBase) -> ::std::borrow::Cow<str>;
        }
    }
}

pub mod imp {
    #[cfg(all(target_os = "macos", feature = "cocoa_"))]
    pub use crate::lib_cocoa::WebView;
    #[cfg(feature = "gtk3")]
    pub use crate::lib_gtk::WebView;
    #[cfg(all(target_os = "windows", feature = "win32"))]
    pub use crate::lib_win32::WebView;
}
