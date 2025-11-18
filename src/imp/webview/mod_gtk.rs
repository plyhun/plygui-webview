use crate::sdk::*;

use plygui_gtk::common::*;
use plygui_gtk::glib::object::Cast;
use webview_sys;

use std::str;
use std::ffi::CStr;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type Webview = AMember<AControl<AWebview<GtkWebview>>>;

#[repr(C)]
pub struct GtkWebview {
    base: GtkControlBase<Webview>,
    webview_wrapper: *mut c_void,
    bindings: HashMap<String, Box<[*mut c_void; 3]>>,
}

impl<O: crate::Webview> NewWebviewInner<O> for GtkWebview {
    fn with_uninit(u: &mut mem::MaybeUninit<O>) -> Self {
        use crate::plygui_gtk::glib::translate::FromGlibPtrFull;

        let webview_wrapper = unsafe { webview_sys::webview_create_control(0) };
        let mut sc = Self {
            base: GtkControlBase::with_gtk_widget(unsafe { 
                Widget::from_glib_full(mem::transmute(webview_sys::webview_get_native_handle(webview_wrapper, webview_sys::webview_native_handle_kind_t_WEBVIEW_NATIVE_HANDLE_KIND_UI_WIDGET))) 
            }),
            webview_wrapper,
            bindings: HashMap::new(),
        };
        {
            let ptr = u as *mut _ as *mut c_void;
            sc.base.set_pointer(ptr);
        }
        Object::from(sc.base.widget.clone()).downcast::<::plygui_gtk::gtk::Widget>().unwrap().connect_size_allocate(on_size_allocate::<O>);
        sc
    }
}
impl WebviewInner for GtkWebview {
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
impl WebviewExtInner for GtkWebview {
    type W = Webview;
    fn bind<C, F>(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, name: Cow<str>, context: Arc<RwLock<C>>, callback: F) -> Result<(), WebviewError> 
            where F: FnMut(&mut Self::W, &str, &str, &mut C), C: WebviewBindContext {
        let widget: Object = Object::from(self.base.widget.clone()).into();
        let gobject: *mut GObject = widget.to_glib_full();
        let context = Arc::into_raw(context) as *const _ as *mut c_void;
        let callback = Box::new(callback);
        let inner_context = Box::new([
            gobject as *mut c_void, 
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
                    use crate::plygui_gtk::glib::translate::FromGlibPtrFull;
                    let arg = &*(arg as *const [*mut c_void; 3]);
                    let mut object = Object::from_glib_full(arg[0] as *mut GObject);
                    let this: &mut Webview = cast_gobject_mut(&mut object).expect("Not a GTK Control");
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

impl HasLayoutInner for GtkWebview {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl ControlInner for GtkWebview {
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

impl HasNativeIdInner for GtkWebview {
    type Id = GtkWidget;

    fn native_id(&self) -> Self::Id {
        self.base.widget.clone().into()
    }
}

impl HasSizeInner for GtkWebview {
    fn on_size_set(&mut self, _: &mut MemberBase, (width, height): (u16, u16)) -> bool {
        self.base.widget().set_size_request(width as i32, height as i32);
        true
    }
}

impl HasVisibilityInner for GtkWebview {
    fn on_visibility_set(&mut self, _: &mut MemberBase, _: types::Visibility) -> bool {
        self.base.invalidate()
    }
}

impl MemberInner for GtkWebview {}

impl Drawable for GtkWebview {
    fn draw(&mut self, _: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(control);
    }
    fn measure(&mut self, _: &mut MemberBase, control: &mut ControlBase, w: u16, h: u16) -> (u16, u16, bool) {
        let old_size = control.measured;
        let (lm, tm, rm, bm) = self.base.margins().into();

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
                (cmp::max(0, w as i32 + lm + rm) as u16, cmp::max(0, h as i32 + tm + bm) as u16)
            }
        };
        (control.measured.0, control.measured.1, control.measured != old_size)
    }
    fn invalidate(&mut self, _: &mut MemberBase, _: &mut ControlBase) {
        self.base.invalidate();
    }
}
impl Spawnable for GtkWebview {
    fn spawn() -> Box<dyn controls::Control> {
        Self::new().into_control()
    }
}
fn on_size_allocate<O: crate::Webview>(this: &::plygui_gtk::gtk::Widget, _allo: &::plygui_gtk::gtk::Rectangle) {
    use plygui_api::controls::HasSize;

    let mut ll = this.clone().upcast::<::plygui_gtk::gtk::Widget>();
    let ll = cast_gtk_widget_to_member_mut::<Webview>(&mut ll).unwrap();

    let measured_size = ll.size();
    ll.call_on_size::<O>(measured_size.0 as u16, measured_size.1 as u16);
}
