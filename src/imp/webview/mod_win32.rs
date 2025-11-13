use crate::sdk::*;

use plygui_win32::common::*;
use webview_sys::{Webview_RegisterClasses, Webview_ReleaseResources};

use std::os::raw::{c_int, c_long, c_ulong, c_void as r_void};
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicUsize;

static GLOBAL_COUNT: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = OsStr::new("PlyguiWebview").encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
}

pub type Webview = AMember<AControl<AWebview<WindowsWebview>>>;

#[repr(C)]
pub struct WindowsWebview {
    base: WindowsControlBase<Webview>
}
impl<O: crate::Webview> NewWebviewInner<O> for WindowsWebview {
    fn with_uninit(_: &mut mem::MaybeUninit<O>) -> Self {
        if GLOBAL_COUNT.fetch_add(1, Ordering::SeqCst) < 1 {
            unsafe {
                if Webview_RegisterClasses(hinstance() as *mut r_void) == 0 {
                    panic!("Cannot register Webview Win32 class");
                }
            }
        }
		Self {
            base: WindowsControlBase::with_handler(Some(handler::<O>)),
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
    fn navigate(&mut self, url: &str) -> Result<(), WebviewError>;
    fn set_html(&mut self, html: &str) -> Result<(), WebviewError>;
    fn init(&mut self, js: &str) -> Result<(), WebviewError>;
    fn eval(&mut self, js: &str) -> Result<(), WebviewError>;
    fn bind(&mut self, name: &str, callback: F) where F: FnMut(&mut dyn Webview, &str) -> Result<(), WebviewError>;
    fn unbind(&mut self, name: &str) -> Result<(), WebviewError>;
    fn return_(&mut self, id: &str, status: i32, result: &str) -> Result<(), WebviewError>;
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
        self.base.create_control_hwnd(x as i32, y as i32, w as i32, h as i32, unsafe { parent.native_id() as windef::HWND }, 0, WINDOW_CLASS.as_ptr(), "", winuser::BS_PUSHBUTTON | winuser::WS_TABSTOP, selfptr);
    
        unsafe {
            self.fn_ptr = Some(mem::transmute(winuser::SendMessageW(self.base.hwnd, crate::webview_sys::SCI_GETDIRECTFUNCTION, 0, 0)));
            self.self_ptr = Some(winuser::SendMessageW(self.base.hwnd, crate::webview_sys::SCI_GETDIRECTPOINTER, 0, 0) as *mut r_void);
        }
    }
    fn on_removed_from_container(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, _: &dyn controls::Container) {
        self.base.destroy_control_hwnd();
        self.fn_ptr = None;
        self.self_ptr = None;
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

impl Drop for WindowsWebview {
    fn drop(&mut self) {
        if GLOBAL_COUNT.fetch_sub(1, Ordering::SeqCst) < 1 {
            unsafe {
                Webview_ReleaseResources();
            }
        }
    }
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
