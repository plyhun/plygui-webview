use plygui_api::{
    controls::{Member, Control},
    sdk::{AControl, ControlInner, HasInner, AMember, Abstract, MemberBase, ControlBase},
};

use webview_sys;
use std::borrow::Cow;
use std::sync::{Arc, RwLock};

pub enum WebviewError {
	MissingDependency,
	Canceled,
	InvalidState,
	InvalidArgument,
	Unspecified(i32),
	Duplicate,
	NotFound
}

impl WebviewError {
	pub(crate) fn from_native(native: i32) -> Result<(), WebviewError> {
		match native {
			webview_sys::webview_error_t_WEBVIEW_ERROR_OK => Result::Ok(()),
			webview_sys::webview_error_t_WEBVIEW_ERROR_DUPLICATE => Result::Err(WebviewError::Duplicate),
			webview_sys::webview_error_t_WEBVIEW_ERROR_NOT_FOUND => Result::Err(WebviewError::NotFound),
			webview_sys::webview_error_t_WEBVIEW_ERROR_INVALID_ARGUMENT => Result::Err(WebviewError::InvalidArgument),
			webview_sys::webview_error_t_WEBVIEW_ERROR_INVALID_STATE => Result::Err(WebviewError::InvalidState),
			webview_sys::webview_error_t_WEBVIEW_ERROR_CANCELED => Result::Err(WebviewError::Canceled),
			webview_sys::webview_error_t_WEBVIEW_ERROR_MISSING_DEPENDENCY => Result::Err(WebviewError::MissingDependency),
			_ => Result::Err(WebviewError::Unspecified(native)),
		}
	}
}

define! {
    Webview: Control {
        outer: {
            fn navigate(&mut self, url: Cow<str>) -> Result<(), WebviewError>;
            fn set_html(&mut self, html: Cow<str>) -> Result<(), WebviewError>;
            fn init(&mut self, js: Cow<str>) -> Result<(), WebviewError>;
			fn eval(&mut self, js: Cow<str>) -> Result<(), WebviewError>;
        }
        inner: {
            fn navigate(&mut self, member: &mut MemberBase, control: &mut ControlBase, url: Cow<str>) -> Result<(), WebviewError>;
            fn set_html(&mut self, member: &mut MemberBase, control: &mut ControlBase, html: Cow<str>) -> Result<(), WebviewError>;
            fn init(&mut self, member: &mut MemberBase, control: &mut ControlBase, js: Cow<str>) -> Result<(), WebviewError>;
			fn eval(&mut self, member: &mut MemberBase, control: &mut ControlBase, js: Cow<str>) -> Result<(), WebviewError>;
        }
        constructor: {
            fn new() -> Box<dyn Webview>;
        }
    }
}
impl<II: WebviewInner, T: HasInner<I = II> + Abstract + 'static> WebviewInner for T {
    default fn new() -> Box<dyn Webview> {
        <<Self as HasInner>::I as WebviewInner>::new()
    }
	default fn navigate(&mut self, member: &mut MemberBase, control: &mut ControlBase, url: Cow<str>) -> Result<(), WebviewError> {
		self.inner_mut().navigate(member, control, url)
	}
	default fn set_html(&mut self, member: &mut MemberBase, control: &mut ControlBase, html: Cow<str>) -> Result<(), WebviewError> {
		self.inner_mut().set_html(member, control, html)
	}
	default fn init(&mut self, member: &mut MemberBase, control: &mut ControlBase, js: Cow<str>) -> Result<(), WebviewError> {
		self.inner_mut().init(member, control, js)
	}
	default fn eval(&mut self, member: &mut MemberBase, control: &mut ControlBase, js: Cow<str>) -> Result<(), WebviewError> {
		self.inner_mut().eval(member, control, js)
	}
}
impl<T: WebviewInner> Webview for AMember<AControl<AWebview<T>>> {
    default fn navigate(&mut self, url: Cow<str>) -> Result<(), WebviewError> {
		self.inner.inner.inner.navigate(&mut self.base, &mut self.inner.base, url)
	}
	default fn set_html(&mut self, html: Cow<str>) -> Result<(), WebviewError> {
		self.inner.inner.inner.set_html(&mut self.base, &mut self.inner.base, html)
	}
	default fn init(&mut self, js: Cow<str>) -> Result<(), WebviewError> {
		self.inner.inner.inner.init(&mut self.base, &mut self.inner.base, js)
	}
	default fn eval(&mut self, js: Cow<str>) -> Result<(), WebviewError> {
		self.inner.inner.inner.eval(&mut self.base, &mut self.inner.base, js)
	}
    default fn as_webview(& self) -> & dyn Webview { self } 
    default fn as_webview_mut (& mut self) -> & mut dyn Webview { self } 
    default fn into_webview (self : Box < Self >) -> Box < dyn Webview > { self }
}
impl<T: WebviewInner> NewWebview for AMember<AControl<AWebview<T>>> {
    fn new() -> Box<dyn Webview> {
        T::new()
    }
}
pub trait WebviewBindContext: Send + Sized {}

pub trait WebviewExt: Webview {
	fn bind<C, F>(&mut self, name: Cow<str>, context: Arc<RwLock<C>>, callback: F) -> Result<(), WebviewError> where C: WebviewBindContext, F: FnMut(&mut Self, &str, &str, &mut C);
	fn unbind(&mut self, name: Cow<str>) -> Result<(), WebviewError>;
	fn return_(&mut self, id: Cow<str>, status: i32, result: Cow<str>) -> Result<(), WebviewError>;
}
pub trait WebviewExtInner: WebviewInner {
	type W: WebviewExt;
	fn bind<C, F>(&mut self, member: &mut MemberBase, control: &mut ControlBase, name: Cow<str>, context: Arc<RwLock<C>>, callback: F) -> Result<(), WebviewError> 
		where C: WebviewBindContext, F: FnMut(&mut Self::W, &str, &str, &mut C);
	fn unbind(&mut self, member: &mut MemberBase, control: &mut ControlBase, name: Cow<str>) -> Result<(), WebviewError>;
	fn return_(&mut self, member: &mut MemberBase, control: &mut ControlBase, id: Cow<str>, status: i32, result: Cow<str>) -> Result<(), WebviewError>;
}
impl<WW: WebviewExt, II: WebviewExtInner<W=WW>, T: HasInner<I = II> + Abstract + 'static> WebviewExtInner for T {
	type W = WW;
	default fn bind<C, F>(&mut self, member: &mut MemberBase, control: &mut ControlBase, name: Cow<str>, context: Arc<RwLock<C>>, callback: F) -> Result<(), WebviewError> 
			where C: WebviewBindContext, F: FnMut(&mut Self::W, &str, &str, &mut C) {
		self.inner_mut().bind(member, control, name, context, callback)
	}
	default fn unbind(&mut self, member: &mut MemberBase, control: &mut ControlBase, name: Cow<str>) -> Result<(), WebviewError> {
		self.inner_mut().unbind(member, control, name)
	}
	default fn return_(&mut self, member: &mut MemberBase, control: &mut ControlBase, id: Cow<str>, status: i32, result: Cow<str>) -> Result<(), WebviewError> {
		self.inner_mut().return_(member, control, id, status, result)
	}
}
impl<T: WebviewExtInner<W=Self>> WebviewExt for AMember<AControl<AWebview<T>>> {
    default fn bind<C, F>(&mut self, name: Cow<str>, context: Arc<RwLock<C>>, callback: F) -> Result<(), WebviewError> where C: WebviewBindContext, F: FnMut(&mut Self, &str, &str, &mut C) {
		self.inner.inner.inner.bind(&mut self.base, &mut self.inner.base, name, context, callback)
	}
	default fn unbind(&mut self, name: Cow<str>) -> Result<(), WebviewError> {
		self.inner.inner.inner.unbind(&mut self.base, &mut self.inner.base, name)
	}
	default fn return_(&mut self, id: Cow<str>, status: i32, result: Cow<str>) -> Result<(), WebviewError> {
		self.inner.inner.inner.return_(&mut self.base, &mut self.inner.base, id, status, result)
	}
}