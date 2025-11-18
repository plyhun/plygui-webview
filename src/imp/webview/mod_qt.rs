use crate::sdk::*;

use plygui_qt::common::{self, *};
use webview_sys;

use std::str;
use std::ffi::CStr;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type Webview = AMember<AControl<AWebview<QtWebview>>>;

#[repr(C)]
pub struct QtWebview {
    base: QtControlBase<Webview, QWidget>,
    webview_wrapper: *mut c_void,
    bindings: HashMap<String, Box<[*mut c_void; 3]>>,
}

impl<O: crate::Webview> NewWebviewInner<O> for QtWebview {
    fn with_uninit(u: &mut mem::MaybeUninit<O>) -> Self {
        let webview_wrapper = unsafe { webview_sys::webview_create_control(0) };
        let sc = Self {
            base: QtControlBase::with_params(unsafe { 
                QBox::from_raw(mem::transmute(webview_sys::webview_get_native_handle(webview_wrapper, webview_sys::webview_native_handle_kind_t_WEBVIEW_NATIVE_HANDLE_KIND_UI_WIDGET))) 
            }, event_handler::<O>),
            webview_wrapper,
            bindings: HashMap::new(),
        };
        unsafe {
            let ptr = u as *mut _ as u64;
            let qo: &QObject = &mut sc.base.widget.static_upcast();
            qo.set_property(common::PROPERTY.as_ptr() as *const i8, &QVariant::from_u64(ptr));
        }
        sc
    }
}
impl WebviewInner for QtWebview {
    fn new() -> Box<dyn crate::Webview> {        
        let mut b: Box<mem::MaybeUninit<Webview>> = Box::new_uninit();
        let ab = AMember::with_inner(
            AControl::with_inner(
                AWebview::with_inner(
                    <Self as NewWebviewInner<Webview>>::with_uninit(b.as_mut()),
                )
            ),
        );
        unsafe {
	        b.as_mut_ptr().write(ab);
	        b.assume_init()
        }
    }
    fn navigate(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, url: Cow<str>) -> Result<(), WebviewError> {
        unsafe {
            let c_url = CString::new(&*url).map_err(|_| WebviewError::InvalidArgument)?;
            let err_code = webview_sys::webview_navigate(self.webview_wrapper, c_url.as_ptr());
            WebviewError::from_native(err_code)
        }
    }
    fn set_html(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, html: Cow<str>) -> Result<(), WebviewError> {
        unsafe {
            let c_html = CString::new(&*html).map_err(|_| WebviewError::InvalidArgument)?;
            let err_code = webview_sys::webview_set_html(self.webview_wrapper, c_html.as_ptr());
            WebviewError::from_native(err_code)
        }
    }
    fn init(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, js: Cow<str>) -> Result<(), WebviewError> {
        unsafe {
            let c_js = CString::new(&*js).map_err(|_| WebviewError::InvalidArgument)?;
            let err_code = webview_sys::webview_init(self.webview_wrapper, c_js.as_ptr());
            WebviewError::from_native(err_code)
        }
    }
    fn eval(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, js: Cow<str>) -> Result<(), WebviewError> {
        unsafe {
            let c_js = CString::new(&*js).map_err(|_| WebviewError::InvalidArgument)?;
            let err_code = webview_sys::webview_eval(self.webview_wrapper, c_js.as_ptr());
            WebviewError::from_native(err_code)
        }
    }    
    fn url(&self, _member: &MemberBase, _control: &ControlBase) -> Result<Cow<str>,WebviewError> {
        unsafe {
            let c_url = webview_sys::webview_get_url(self.webview_wrapper);
            if c_url.is_null() {
                return Err(WebviewError::NotFound);
            }
            let url = CStr::from_ptr(c_url).to_str().map_err(|_| WebviewError::InvalidArgument)?;
            Ok(Cow::Owned(url.to_string()))
        }
    }    
    fn title(&self, _member: &MemberBase, _control: &ControlBase) -> Result<Cow<str>,WebviewError> {
        unsafe {
            let c_title = webview_sys::webview_get_title(self.webview_wrapper);
            if c_title.is_null() {
                return Err(WebviewError::NotFound);
            }
            let url = CStr::from_ptr(c_title).to_str().map_err(|_| WebviewError::InvalidArgument)?;
            Ok(Cow::Owned(url.to_string()))
        }
    }    
    fn back(&mut self, _member: &mut MemberBase, _control: &mut ControlBase) -> Result<(),WebviewError> {
        unsafe {
            let err_code = webview_sys::webview_go_back(self.webview_wrapper);
            WebviewError::from_native(err_code)
        }
    }    
    fn forward(&mut self, _member: &mut MemberBase, _control: &mut ControlBase) -> Result<(),WebviewError> {
        unsafe {
            let err_code = webview_sys::webview_go_forward(self.webview_wrapper);
            WebviewError::from_native(err_code)
        }
    }    
    fn stop(&mut self, _member: &mut MemberBase, _control: &mut ControlBase) -> Result<(),WebviewError> {
        unsafe {
            let err_code = webview_sys::webview_stop(self.webview_wrapper);
            WebviewError::from_native(err_code)
        }
    }    
    fn reload(&mut self, _member: &mut MemberBase, _control: &mut ControlBase) -> Result<(),WebviewError> {
        unsafe {
            let err_code = webview_sys::webview_reload(self.webview_wrapper);
            WebviewError::from_native(err_code)
        }
    }
}
impl WebviewExtInner for QtWebview {
    type W = Webview;
    fn bind<C, F>(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, name: Cow<str>, context: Arc<RwLock<C>>, callback: F) -> Result<(), WebviewError> 
            where F: FnMut(&mut Self::W, &str, &str, &mut C), C: WebviewBindContext {
        let qwidget = unsafe { self.base.as_qwidget().as_ptr().as_raw_ptr() };
        let context = Arc::into_raw(context) as *const _ as *mut c_void;
        let callback = Box::new(callback);
        let inner_context = Box::new([
            qwidget as *mut c_void, 
            context, 
            Box::into_raw(callback) as *mut c_void
        ]);
        extern "C" fn trampoline_webview_bind<
                F: FnMut(&mut Webview, &str, &str, &mut CC),
                CC: WebviewBindContext
            >(
                id: *const ::std::os::raw::c_char,
                req: *const ::std::os::raw::c_char,
                arg: *mut ::std::os::raw::c_void,
            ) {
                unsafe {
                    let arg = &*(arg as *const [*mut c_void; 3]);
                    let object: *mut QWidget = mem::transmute(arg[0]);
                    let this: &mut Webview = cast_qobject_mut(&*object).expect("Not a Qt Control");
                    let id = CStr::from_ptr(id).to_str().expect("id is not a valid string");
                    let req = CStr::from_ptr(req).to_str().expect("req is not a valid string");
                    let context: &mut RwLock<CC> = mem::transmute(arg[1]);
                    let callback: &mut F = mem::transmute(arg[2]);
                    callback(this, id, req, &mut context.write().unwrap());
                    mem::forget(object);
                }
            }
            unsafe {
                let c_name = CString::new(&*name).map_err(|_| WebviewError::InvalidArgument)?;
                let err_code = webview_sys::webview_bind(
                    self.webview_wrapper, 
                    c_name.as_ptr(), 
                    Some(trampoline_webview_bind::<F, C>), 
                    inner_context.as_ptr() as *const _ as *mut c_void
                );
                let res = WebviewError::from_native(err_code);
                match res {
                    Err(_) => {
                        self.bindings.remove(&name.to_string()).map(|binding| {
                            let _ = Arc::from_raw(binding[1]);
                            let _ = Box::from_raw(binding[2]);
                        });                    
                    },
                    Ok(_) => {
                        self.bindings.insert(name.to_string(), inner_context);
                    }
                }
                res
            }
    }
    fn unbind(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, name: Cow<str>) -> Result<(), WebviewError> {
        unsafe {
            let c_name = CString::new(&*name).map_err(|_| WebviewError::InvalidArgument)?;
            let err_code = webview_sys::webview_unbind(self.webview_wrapper, c_name.as_ptr());
            let res = WebviewError::from_native(err_code);
                match res {
                    Ok(_) => {
                        self.bindings.remove(&name.to_string()).map(|binding| {
                            let _ = Box::from_raw(binding[0]);
                            let _ = Box::from_raw(binding[1]);
                        });                    
                    },
                    _ => {}
                }
                res
        }
    }
    fn return_(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, id: Cow<str>, status: i32, result: Cow<str>) -> Result<(), WebviewError> {
        unsafe {
            let c_id = CString::new(&*id).map_err(|_| WebviewError::InvalidArgument)?;
            let c_result = CString::new(&*result).map_err(|_| WebviewError::InvalidArgument)?;
            let err_code = webview_sys::webview_return(self.webview_wrapper, c_id.as_ptr(), status, c_result.as_ptr());
            WebviewError::from_native(err_code)
        }
    }
}

impl HasLayoutInner for QtWebview {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl ControlInner for QtWebview {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, _parent: &dyn controls::Container, x: i32, y: i32, pw: u16, ph: u16) {
        self.measure(member, control, pw, ph);
        control.coords = Some((x, y));
        self.draw(member, control);
    }
    fn on_removed_from_container(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &dyn controls::Container) {}

    fn parent(&self) -> Option<&dyn controls::Member> {
        self.base.parent().map(|m| m.as_member())
    }
    fn parent_mut(&mut self) -> Option<&mut dyn controls::Member> {
        self.base.parent_mut().map(|m| m.as_member_mut())
    }
    fn root(&self) -> Option<&dyn controls::Member> {
        self.base.root().map(|m| m.as_member())
    }
    fn root_mut(&mut self) -> Option<&mut dyn controls::Member> {
        self.base.root_mut().map(|m| m.as_member_mut())
    }
}

impl HasNativeIdInner for QtWebview {
    type Id = common::QtId;

    fn native_id(&self) -> Self::Id {
        QtId::from(unsafe { self.base.widget.static_upcast::<QObject>().as_raw_ptr() } as *mut QObject)
    }
}

impl HasSizeInner for QtWebview {
    fn on_size_set(&mut self, _: &mut MemberBase, (width, height): (u16, u16)) -> bool {
        unsafe { self.base.widget.set_fixed_size_2a(width as i32, height as i32); }
        true
    }
}

impl HasVisibilityInner for QtWebview {
    fn on_visibility_set(&mut self, _: &mut MemberBase, _: types::Visibility) -> bool {
        self.base.invalidate()
    }
}

impl MemberInner for QtWebview {}

impl Drawable for QtWebview {
    fn draw(&mut self, member_base: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(member_base, control);
    }
    fn measure(&mut self, _: &mut MemberBase, control: &mut ControlBase, w: u16, h: u16) -> (u16, u16, bool) {
        let old_size = control.measured;

        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let w = match control.layout.width {
                    layout::Size::MatchParent => w,
                    layout::Size::Exact(w) => w,
                    layout::Size::WrapContent => {
                        42 as u16 // TODO min_width
                    }
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => h,
                    layout::Size::Exact(h) => h,
                    layout::Size::WrapContent => {
                        42 as u16 // TODO min_height
                    }
                };
                (cmp::max(0, w) as u16, cmp::max(0, h) as u16)
            }
        };
        (control.measured.0, control.measured.1, control.measured != old_size)
    }
    fn invalidate(&mut self, _: &mut MemberBase, _: &mut ControlBase) {
        self.base.invalidate();
    }
}
impl Spawnable for QtWebview {
    fn spawn() -> Box<dyn controls::Control> {
        Self::new().into_control()
    }
}

fn event_handler<O: crate::Webview>(object: &mut QObject, event: &mut QEvent) -> bool {
    match unsafe { event.type_() } {
        QEventType::Resize => {
            if let Some(this) = cast_qobject_to_uimember_mut::<Webview>(object) {
                let size = unsafe { 
                    let size = Ptr::from_raw(event).static_downcast::<QResizeEvent>();
                    let size = (
                    	utils::coord_to_size(size.size().width()), 
                    	utils::coord_to_size(size.size().height())
                    );
                    this.inner_mut().base.measured = size;
                    if let layout::Size::WrapContent = this.inner_mut().base.layout.width {
                        this.inner_mut().inner_mut().inner_mut().base.widget.set_minimum_width(size.0 as i32);  
                        this.inner_mut().inner_mut().inner_mut().base.widget.set_maximum_width(size.0 as i32); 
                    }
                    if let layout::Size::WrapContent = this.inner_mut().base.layout.height {
                        this.inner_mut().inner_mut().inner_mut().base.widget.set_minimum_height(size.1 as i32); 
                        this.inner_mut().inner_mut().inner_mut().base.widget.set_maximum_height(size.1 as i32); 
                    }
                    size
                };
                this.call_on_size::<O>(size.0, size.1);
            }
        }
        _ => {}
    }
    false
}