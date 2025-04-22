use crate::sdk::*;

use plygui_win32::common::*;
use scintilla_sys::{Scintilla_RegisterClasses, Scintilla_ReleaseResources};

use std::os::raw::{c_int, c_long, c_ulong, c_void as r_void};
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicUsize;

static GLOBAL_COUNT: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = OsStr::new("Scintilla").encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
}

pub type Scintilla = AMember<AControl<AScintilla<WindowsScintilla>>>;

#[repr(C)]
pub struct WindowsScintilla {
    base: WindowsControlBase<Scintilla>,

    fn_ptr: Option<extern "C" fn(*mut r_void, c_int, c_ulong, c_long) -> *mut r_void>,
    self_ptr: Option<*mut r_void>,
}
impl<O: crate::Scintilla> NewScintillaInner<O> for WindowsScintilla {
    fn with_uninit(_: &mut mem::MaybeUninit<O>) -> Self {
        if GLOBAL_COUNT.fetch_add(1, Ordering::SeqCst) < 1 {
            unsafe {
                if Scintilla_RegisterClasses(hinstance() as *mut r_void) == 0 {
                    panic!("Cannot register Scintilla Win32 class");
                }
            }
        }
		Self {
            base: WindowsControlBase::with_handler(Some(handler::<O>)),
            fn_ptr: None,
            self_ptr: None,
        }
    }
}
impl ScintillaInner for WindowsScintilla {
    fn new() -> Box<dyn crate::Scintilla> {        
        let mut b: Box<mem::MaybeUninit<Scintilla>> = Box::new_uninit();
        let ab = AMember::with_inner(
            AControl::with_inner(
                AScintilla::with_inner(
                    <Self as NewScintillaInner<Scintilla>>::with_uninit(b.as_mut()),
                )
            ),
        );
        unsafe {
	        b.as_mut_ptr().write(ab);
	        b.assume_init()
        }
    }
    fn set_margin_width(&mut self, index: usize, width: isize) {
        if let Some(fn_ptr) = self.fn_ptr {
            (fn_ptr)(self.self_ptr.unwrap(), crate::scintilla_sys::SCI_SETMARGINWIDTHN as i32, index as c_ulong, width as c_long);
        }
    }
    fn set_readonly(&mut self, readonly: bool) {
        if let Some(fn_ptr) = self.fn_ptr {
            (fn_ptr)(self.self_ptr.unwrap(), crate::scintilla_sys::SCI_SETREADONLY as i32, if readonly { 1 } else { 0 }, 0);
        }
    }
    fn is_readonly(&self) -> bool {
        if let Some(fn_ptr) = self.fn_ptr {
            !(fn_ptr)(self.self_ptr.unwrap(), crate::scintilla_sys::SCI_GETREADONLY as i32, 0, 0).is_null()
        } else {
            true
        }
    }
    fn set_codepage(&mut self, cp: crate::Codepage) {
        if let Some(fn_ptr) = self.fn_ptr {
            ((fn_ptr)(self.self_ptr.unwrap(), crate::scintilla_sys::SCI_SETCODEPAGE as i32, cp as c_ulong, 0) as isize);
        }
    }
    fn codepage(&self) -> crate::Codepage {
        if let Some(fn_ptr) = self.fn_ptr {
            ((fn_ptr)(self.self_ptr.unwrap(), crate::scintilla_sys::SCI_GETCODEPAGE as i32, 0, 0) as isize).into()
        } else {
            Default::default()
        }
    }
    fn append_text(&mut self, text: &str) {
        self.set_codepage(crate::Codepage::Utf8);
        if let Some(fn_ptr) = self.fn_ptr {
            let len = text.len();
            let tptr = text.as_bytes().as_ptr();
            (fn_ptr)(self.self_ptr.unwrap(), crate::scintilla_sys::SCI_APPENDTEXT as i32, len as c_ulong, tptr as c_long);
        }
    }
}

impl Spawnable for WindowsScintilla {
    fn spawn() -> Box<dyn controls::Control> {
        Self::new().into_control()
    }
}
impl ControlInner for WindowsScintilla {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent: &dyn controls::Container, x: i32, y: i32, pw: u16, ph: u16) {
        let selfptr = member as *mut _ as *mut c_void;
        self.base.hwnd = unsafe { parent.native_id() as windef::HWND }; // required for measure, as we don't have own hwnd yet
        let (w, h, _) = self.measure(member, control, pw, ph);
        self.base.create_control_hwnd(x as i32, y as i32, w as i32, h as i32, unsafe { parent.native_id() as windef::HWND }, 0, WINDOW_CLASS.as_ptr(), "", winuser::BS_PUSHBUTTON | winuser::WS_TABSTOP, selfptr);
    
        unsafe {
            self.fn_ptr = Some(mem::transmute(winuser::SendMessageW(self.base.hwnd, crate::scintilla_sys::SCI_GETDIRECTFUNCTION, 0, 0)));
            self.self_ptr = Some(winuser::SendMessageW(self.base.hwnd, crate::scintilla_sys::SCI_GETDIRECTPOINTER, 0, 0) as *mut r_void);
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
        fill_from_markup_base!(self, base, markup, registry, Scintilla, ["Scintilla"]);
    }*/
}

impl Drop for WindowsScintilla {
    fn drop(&mut self) {
        if GLOBAL_COUNT.fetch_sub(1, Ordering::SeqCst) < 1 {
            unsafe {
                Scintilla_ReleaseResources();
            }
        }
    }
}

impl HasLayoutInner for WindowsScintilla {
    fn on_layout_changed(&mut self, _base: &mut MemberBase) {
        let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
            self.base.invalidate();
        }
    }
}

impl HasNativeIdInner for WindowsScintilla {
    type Id = Hwnd;

    fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}

impl HasSizeInner for WindowsScintilla {
    fn on_size_set(&mut self, base: &mut MemberBase, (width, height): (u16, u16)) -> bool {
        use plygui_api::controls::HasLayout;

        let this = base.as_any_mut().downcast_mut::<Scintilla>().unwrap();
        this.set_layout_width(layout::Size::Exact(width));
        this.set_layout_width(layout::Size::Exact(height));
        self.base.invalidate();
        true
    }
}
impl HasVisibilityInner for WindowsScintilla {
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

impl MemberInner for WindowsScintilla {}

impl Drawable for WindowsScintilla {
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

unsafe extern "system" fn handler<T: crate::Scintilla>(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM, _: usize, param: usize) -> isize {
    let sc: &mut Scintilla = mem::transmute(param);
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
