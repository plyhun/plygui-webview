use plygui_api::{
    controls::{Member, Control},
    sdk::{AControl, ControlInner, HasInner, AMember, Abstract, ControlBase, MemberBase},
};

define! {
    WebView: Control {
        outer: {
            fn set_url(&mut self, url: &str);
            fn url(&self) -> ::std::borrow::Cow<str>;
        }
        inner: {
            fn set_url(&mut self, member: &mut MemberBase, control: &mut ControlBase, url: &str);
            fn url(&self) -> ::std::borrow::Cow<str>;
        }
        constructor: {
            fn new() -> Box<dyn WebView>;
        }
    }
}
impl<II: WebViewInner, T: HasInner<I = II> + Abstract + 'static> WebViewInner for T {
    default fn new() -> Box<dyn WebView> {
        <<Self as HasInner>::I as WebViewInner>::new()
    }
    default fn set_url(&mut self, member: &mut MemberBase, control: &mut ControlBase, url: &str) {
        self.inner_mut().set_url(member, control, url)
    }
    default fn url(&self) -> ::std::borrow::Cow<str> {
        self.inner().url()
    }
}
impl<T: WebViewInner> WebView for AMember<AControl<AWebView<T>>> {
    default fn set_url(&mut self, url: &str) {
        let (m,c,t) = self.as_control_parts_mut();
        t.set_url(m, c, url)
    }
    default fn url(&self) -> ::std::borrow::Cow<str> {
        self.inner().inner().inner().url()
    }
    default fn as_web_view(& self) -> & dyn WebView { self } 
    default fn as_web_view_mut (& mut self) -> & mut dyn WebView { self } 
    default fn into_web_view (self : Box < Self >) -> Box < dyn WebView > { self }
}

impl<T: WebViewInner> NewWebView for AMember<AControl<AWebView<T>>> {
    fn new() -> Box<dyn WebView> {
        T::new()
    }
}
