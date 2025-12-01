use crate::sdk::*;

use plygui_win32::common::windef::HWND;
use plygui_win32::common::*;
use webview_sys;

use std::str;
use std::ffi::CStr;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = OsStr::new("PlyguiWebview").encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
}

pub type Webview = AMember<AControl<AWebview<WindowsWebview>>>;

#[repr(C)]
pub struct WindowsWebview {
    base: WindowsControlBase<Webview>,
    webview_wrapper: webview_sys::webview_t,
    bindings: HashMap<String, Box<[*mut c_void; 3]>>,
}
impl<O: crate::Webview> NewWebviewInner<O> for WindowsWebview {
    fn with_uninit(_: &mut mem::MaybeUninit<O>) -> Self {
        Self {
            base: WindowsControlBase::with_handler(Some(handler::<O>)),
            webview_wrapper: ptr::null_mut(),
            bindings: HashMap::new(),
        }
    }
}
impl WebviewInner for WindowsWebview {
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
impl WebviewExtInner for WindowsWebview {
    type W = Webview;
    fn bind<C, F>(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, name: Cow<str>, context: Arc<RwLock<C>>, callback: F) -> Result<(), WebviewError> 
            where F: FnMut(&mut Self::W, &str, &str, &mut C), C: WebviewBindContext {
        let context = Arc::into_raw(context) as *const _ as *mut c_void;
        let callback = Box::new(callback);
        let inner_context = Box::new([
            self.base.hwnd as *mut c_void, 
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
                    let object = arg[0] as HWND;
                    let this: &mut Webview = member_from_hwnd(object).expect("Not a Win32 Control");
                    let id = CStr::from_ptr(id).to_str().expect("id is not a valid string");
                    let req = CStr::from_ptr(req).to_str().expect("req is not a valid string");
                    let context: &mut RwLock<CC> = mem::transmute(arg[1]);
                    let callback: &mut F = mem::transmute(arg[2]);
                    callback(this, id, req, &mut context.write().unwrap());
                }
            }
            unsafe {
                let c_name = CString::new(&*name).map_err(|_| WebviewError::InvalidArgument)?;
                let err_code = webview_sys::webview_bind(
                    self.webview_wrapper, 
                    c_name.as_ptr(), 
                    Some(trampoline_webview_bind::<F, C>), 
                    inner_context.as_ptr() as *const _ as *mut ::std::ffi::c_void
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
impl Spawnable for WindowsWebview {
    fn spawn() -> Box<dyn controls::Control> {
        Self::new().into_control()
    }
}
impl ControlInner for WindowsWebview {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent: &dyn controls::Container, x: i32, y: i32, pw: u16, ph: u16) {
        let selfptr = member as *mut _ as *mut c_void;
        self.base.hwnd = unsafe { parent.native_id() as windef::HWND }; // required for measure, as we don't have own hwnd yet
        let (w, h, _) = self.measure(member, control, pw, ph);
        if self.webview_wrapper.is_null() {
            unsafe {
                let webview = webview_sys::webview_create(0, parent.native_id() as *mut ::std::ffi::c_void);
                let hwnd = webview_sys::webview_get_native_handle(webview, webview_sys::webview_native_handle_kind_t_WEBVIEW_NATIVE_HANDLE_KIND_UI_WIDGET) as windef::HWND;
                commctrl::SetWindowSubclass(hwnd, self.base.proc_handler.as_handler(), subclass_id(WINDOW_CLASS.as_ptr() as *const u16) as usize, selfptr as usize);
                set_default_font(hwnd);
                winuser::SetWindowPos(hwnd, ptr::null_mut(), x, y, w as i32, h as i32, 0);
                self.base.hwnd = hwnd;
                self.webview_wrapper = webview;
            }
        } else {
            unsafe {
                winuser::SetParent(self.base.hwnd, parent.native_id() as windef::HWND);
            }
        }
    }
    fn on_removed_from_container(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, _: &dyn controls::Container) {
        unsafe {
            webview_sys::webview_destroy(self.webview_wrapper);
        }
        self.base.hwnd = 0 as windef::HWND;
        self.base.subclass_id = 0;
    }

    fn parent(&self) -> Option<&dyn controls::Member> {
        self.base.parent().map(|p| p.as_member())
    }
    fn parent_mut(&mut self) -> Option<&mut dyn controls::Member> {
        self.base.parent_mut().map(|p| p.as_member_mut())
    }
    fn root(&self) -> Option<&dyn controls::Member> {
        self.base.root().map(|p| p.as_member())
    }
    fn root_mut(&mut self) -> Option<&mut dyn controls::Member> {
        self.base.root_mut().map(|p| p.as_member_mut())
    }

    /*#[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, base: &mut development::MemberControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        fill_from_markup_base!(self, base, markup, registry, Webview, ["Webview"]);
    }*/
}

impl HasLayoutInner for WindowsWebview {
    fn on_layout_changed(&mut self, _base: &mut MemberBase) {
        let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
            self.base.invalidate();
        }
    }
}

impl HasNativeIdInner for WindowsWebview {
    type Id = Hwnd;

    fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}

impl HasSizeInner for WindowsWebview {
    fn on_size_set(&mut self, base: &mut MemberBase, (width, height): (u16, u16)) -> bool {
        use plygui_api::controls::HasLayout;

        let this = base.as_any_mut().downcast_mut::<Webview>().unwrap();
        this.set_layout_width(layout::Size::Exact(width));
        this.set_layout_width(layout::Size::Exact(height));
        self.base.invalidate();
        true
    }
}
impl HasVisibilityInner for WindowsWebview {
    fn on_visibility_set(&mut self, base: &mut MemberBase, visibility: types::Visibility) -> bool {
        let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
            unsafe {
                winuser::ShowWindow(self.base.hwnd, if visibility == types::Visibility::Visible { winuser::SW_SHOW } else { winuser::SW_HIDE });
            }
            self.on_layout_changed(base);
            true
        } else {
            false
        }
    }
}

impl MemberInner for WindowsWebview {}

impl Drawable for WindowsWebview {
    fn draw(&mut self, _member: &mut MemberBase, control: &mut ControlBase) {
        if let Some((x, y)) = control.coords {
            unsafe {
                winuser::SetWindowPos(self.base.hwnd, ptr::null_mut(), x, y, control.measured.0 as i32, control.measured.1 as i32, 0);
            }
        }
    }
    fn measure(&mut self, _member: &mut MemberBase, control: &mut ControlBase, w: u16, h: u16) -> (u16, u16, bool) {
        let old_size = control.measured;
        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let w = match control.layout.width {
                    layout::Size::MatchParent => w,
                    layout::Size::Exact(w) => w,
                    layout::Size::WrapContent => {
                        defaults::THE_ULTIMATE_ANSWER_TO_EVERYTHING // TODO min_width
                    }
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => h,
                    layout::Size::Exact(h) => h,
                    layout::Size::WrapContent => {
                        defaults::THE_ULTIMATE_ANSWER_TO_EVERYTHING // TODO min_height
                    }
                };
                (cmp::max(0, w as i32) as u16, cmp::max(0, h as i32) as u16)
            }
        };
        (control.measured.0, control.measured.1, control.measured != old_size)
    }
    fn invalidate(&mut self, _member: &mut MemberBase, _control: &mut ControlBase) {
        self.base.invalidate()
    }
}

unsafe extern "system" fn handler<T: crate::Webview>(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let sc: &mut Webview = mem::transmute(param);
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, param as WinPtr);
    }
    match msg {
        winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;

            sc.call_on_size::<T>(width, height);            
        }
        _ => {}
    }
    commctrl::DefSubclassProc(hwnd, msg, wparam, lparam)
}
