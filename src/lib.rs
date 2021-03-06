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

pub trait WebView: plygui_api::controls::Control {
    fn set_url(&mut self, site: &str);
    fn url(&self) -> ::std::borrow::Cow<str>;
}

pub trait NewWebView {
    fn new() -> Box<WebView>;
}

pub mod imp {
    #[cfg(all(target_os = "macos", feature = "cocoa_"))]
    pub use crate::lib_cocoa::WebView;
    #[cfg(feature = "gtk3")]
    pub use crate::lib_gtk::WebView;
    #[cfg(all(target_os = "windows", feature = "win32"))]
    pub use crate::lib_win32::WebView;
}

pub mod development {
    use plygui_api::development::*;

    pub trait WebViewInner: ControlInner {
        fn new() -> Box<super::WebView>;
        fn set_url(&mut self, site: &str);
        fn url(&self) -> ::std::borrow::Cow<str>;
    }

    impl<T: WebViewInner + Sized + 'static> super::WebView for Member<Control<T>> {
        fn set_url(&mut self, site: &str) {
            self.as_inner_mut().as_inner_mut().set_url(site)
        }
        fn url<'a>(&'a self) -> ::std::borrow::Cow<'a, str> {
            self.as_inner().as_inner().url()
        }
    }
    impl<T: WebViewInner + Sized> super::NewWebView for Member<Control<T>> {
        fn new() -> Box<super::WebView> {
            T::new()
        }
    }
}
