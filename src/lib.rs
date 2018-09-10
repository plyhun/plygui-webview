#[macro_use]
extern crate plygui_api;

#[cfg(all(target_os = "windows", feature = "win32"))]
#[macro_use]
extern crate lazy_static;
#[cfg(all(target_os = "windows", feature = "win32"))]
mod lib_win32;
#[cfg(all(target_os = "windows", feature = "win32"))]
extern crate plygui_win32;
#[cfg(all(target_os = "windows", feature = "win32"))]
extern crate winapi;
#[cfg(all(target_os = "windows", feature = "win32"))]
use lib_win32 as inner_imp;

#[cfg(feature = "gtk3")]
mod lib_gtk;
#[macro_use]
#[cfg(feature = "gtk3")]
extern crate plygui_gtk;
#[cfg(feature = "gtk3")]
extern crate gtk;
#[cfg(feature = "gtk3")]
extern crate webkit2gtk;
#[cfg(feature = "gtk3")]
use lib_gtk as inner_imp;

#[cfg(all(target_os = "macos", feature = "cocoa_"))]
#[macro_use]
extern crate lazy_static;
#[cfg(all(target_os = "macos", feature = "cocoa_"))]
mod lib_cocoa;
#[cfg(all(target_os = "macos", feature = "cocoa_"))]
#[macro_use]
extern crate objc;
#[cfg(all(target_os = "macos", feature = "cocoa_"))]
extern crate plygui_cocoa;
#[cfg(all(target_os = "macos", feature = "cocoa_"))]
use lib_cocoa as inner_imp;

pub trait WebView: plygui_api::controls::Control {
    fn go_to(&mut self, site: &str);
}

pub trait NewWebView {
    fn new() -> Box<WebView>;
}

pub mod imp {
    pub use inner_imp::WebView;
}

pub mod development {
    use plygui_api::development::*;

    pub trait WebViewInner: ControlInner {
        fn new() -> Box<super::WebView>;
        fn go_to(&mut self, site: &str);
    }

    impl<T: WebViewInner + Sized + 'static> super::WebView for Member<Control<T>> {
        fn go_to(&mut self, site: &str) {
            self.as_inner_mut().as_inner_mut().go_to(site)
        }
    }
    impl<T: WebViewInner + Sized> super::NewWebView for Member<Control<T>> {
        fn new() -> Box<super::WebView> {
            T::new()
        }
    }
}
