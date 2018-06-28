use super::development as webview_dev;

use plygui_api::{layout, types, utils, controls};
use plygui_api::development::*;		
		
use plygui_win32::common;

use winapi::shared::windef;
use winapi::shared::minwindef;
use winapi::um::winuser;
use winapi::um::libloaderapi;
use winapi::ctypes::c_void;

use std::{ptr, mem};
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use std::cmp::max;

#[repr(C)]
struct OleWebView(u8);

lazy_static! {
	pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };
}

pub type WebView = Member<Control<WebViewWin32>>;
extern "C" {
	fn webview_new_with_parent(parent: windef::HWND) -> *mut OleWebView;
	fn webview_delete(thisptr: *mut OleWebView);
	fn webview_navigate(thisptr: *mut OleWebView, sz_url: *const c_void);
	fn webview_set_rect(thisptr: *mut OleWebView, rect: windef::RECT);
}

#[repr(C)]
pub struct WebViewWin32 {
    base: common::WindowsControlBase<WebView>,
    
    oleptr: *mut OleWebView,
}

impl WebViewWin32 {

}

impl Drop for WebViewWin32 {
	fn drop(&mut self) {
		if  !self.oleptr.is_null() {
			unsafe { webview_delete(self.oleptr) };
			self.oleptr = ptr::null_mut();
		}
	}
}

impl webview_dev::WebViewInner for WebViewWin32 {
	fn new() -> Box<super::WebView> {
		let i = Box::new(Member::with_inner(Control::with_inner(WebViewWin32 {
			base: common::WindowsControlBase::new(),
			oleptr: ptr::null_mut(),	
		}, ()), MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut)));
		i
	}
	fn go_to(&mut self, site: &str) {
		let site = OsStr::new(site)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>();
		unsafe {
			webview_navigate(self.oleptr, site.as_ptr() as *const c_void);
		}
	}
}

impl ControlInner for WebViewWin32 {
	fn on_added_to_container(&mut self, base: &mut MemberControlBase, parent: &controls::Container, x: i32, y: i32) {
		let selfptr = base as *mut _ as *mut c_void;
        let (pw, ph) = parent.draw_area_size();
        //let (lp,tp,rp,bp) = self.base.layout.padding.into();
        let (lm, tm, rm, bm) = base.control.layout.margin.into();
        let (hwnd, id) = unsafe {
            self.base.hwnd = parent.native_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
            let (w, h, _) = self.measure(base, pw, ph);
            common::create_control_hwnd(
                x as i32 + lm,
                y as i32 + tm,
                w as i32 - rm - lm,
                h as i32 - bm - tm,
                self.base.hwnd,
                winuser::WS_EX_CONTROLPARENT,
                WINDOW_CLASS.as_ptr(),
                "",
                winuser::WS_TABSTOP,
                selfptr,
                None,
            )
        };
        self.base.hwnd = hwnd;
        self.base.subclass_id = id;
    }
    fn on_removed_from_container(&mut self, _: &mut MemberControlBase, _: &controls::Container) {
        common::destroy_hwnd(self.base.hwnd, self.base.subclass_id, None);
        self.base.hwnd = 0 as windef::HWND;
        self.base.subclass_id = 0;	
    }
    
    fn parent(&self) -> Option<&controls::Member> {
		self.base.parent().map(|p| p.as_member())
	}
    fn parent_mut(&mut self) -> Option<&mut controls::Member> {
    	self.base.parent_mut().map(|p| p.as_member_mut())
    }
    fn root(&self) -> Option<&controls::Member> {
    	self.base.root().map(|p| p.as_member())
    }
    fn root_mut(&mut self) -> Option<&mut controls::Member> {
    	self.base.root_mut().map(|p| p.as_member_mut())
	}
    
    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, base: &mut development::MemberControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
    	fill_from_markup_base!(self, base, markup, registry, WebView, ["WebView"]);
    	//TODO webview source
	}
}

impl HasLayoutInner for WebViewWin32 {
	fn on_layout_changed(&mut self, base: &mut MemberBase) {
		let base = self.cast_base_mut(base);
		self.invalidate(base);
	}
}

impl MemberInner for WebViewWin32 {
	type Id = common::Hwnd;
	
	fn size(&self) -> (u16, u16) {
        let rect = unsafe { common::window_rect(self.base.hwnd) };
        (
            (rect.right - rect.left) as u16,
            (rect.bottom - rect.top) as u16,
        )
    }

    fn on_set_visibility(&mut self, base: &mut MemberBase) {
	    let hwnd = self.base.hwnd;
        if !hwnd.is_null() {
        	unsafe {
	            winuser::ShowWindow(
	                self.base.hwnd,
	                if base.visibility == types::Visibility::Visible {
	                    winuser::SW_SHOW
	                } else {
	                    winuser::SW_HIDE
	                },
	            );
	        }
			self.invalidate(utils::member_control_base_mut(common::member_from_hwnd::<WebView>(hwnd)));
	    }
    }
    unsafe fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}

impl Drawable for WebViewWin32 {
	fn draw(&mut self, base: &mut MemberControlBase, coords: Option<(i32, i32)>) {
		if coords.is_some() {
            self.base.coords = coords;
        }
        let (lm,tm,rm,bm) = base.control.layout.margin.into();
        if let Some((x, y)) = self.base.coords {
            unsafe {
                winuser::SetWindowPos(
                    self.base.hwnd,
                    ptr::null_mut(),
                    x + lm,
                    y + tm,
                    self.base.measured_size.0 as i32 - rm - lm,
                    self.base.measured_size.1 as i32 - bm - tm,
                    0,
                );
            }
        }
	}
    fn measure(&mut self, base: &mut MemberControlBase, w: u16, h: u16) -> (u16, u16, bool) {
    	let old_size = self.base.measured_size;
        let (lp,tp,rp,bp) = base.control.layout.padding.into();
        let (lm, tm, rm, bm) = base.control.layout.margin.into();
        
        self.base.measured_size = match base.member.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let w = match base.control.layout.width {
                    layout::Size::MatchParent => w,
                    layout::Size::Exact(w) => w,
                    layout::Size::WrapContent => {
                        42 as u16 // TODO min_width
                    } 
                };
                let h = match base.control.layout.height {
                    layout::Size::MatchParent => h,
                    layout::Size::Exact(h) => h,
                    layout::Size::WrapContent => {
                        42 as u16 // TODO min_height
                    } 
                };
                (
                    max(0, w as i32 + lm + rm + lp + rp) as u16,
                    max(0, h as i32 + tm + bm + tp + bp) as u16,
                )
            },
        };
        (
            self.base.measured_size.0,
            self.base.measured_size.1,
            self.base.measured_size != old_size,
        )
    }
    fn invalidate(&mut self, base: &mut MemberControlBase) {
    	self.base.invalidate(base)
    }
}

/*
#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
	use super::NewWebView;
	
    WebView::with_content().into_control()
}
*/

unsafe fn register_window_class() -> Vec<u16> {
    let class_name = OsStr::new("PlyguiWin32Browser")
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>();
    let class = winuser::WNDCLASSEXW {
        cbSize: mem::size_of::<winuser::WNDCLASSEXW>() as minwindef::UINT,
        style: winuser::CS_DBLCLKS,
        lpfnWndProc: Some(whandler),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: libloaderapi::GetModuleHandleW(ptr::null()),
        hIcon: winuser::LoadIconW(ptr::null_mut(), winuser::IDI_APPLICATION),
        hCursor: winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_ARROW),
        hbrBackground: ptr::null_mut(),
        lpszMenuName: ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: ptr::null_mut(),
    };
    winuser::RegisterClassExW(&class);
    class_name
}

unsafe extern "system" fn whandler(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM) -> minwindef::LRESULT {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        if winuser::WM_CREATE == msg {
            let cs: &winuser::CREATESTRUCTW = mem::transmute(lparam);
            winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, cs.lpCreateParams as isize);
            let sc: &mut WebView = mem::transmute(cs.lpCreateParams);
		    sc.as_inner_mut().as_inner_mut().oleptr = webview_new_with_parent(hwnd);
		}
        return winuser::DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    
    let sc: &mut WebView = mem::transmute(ww);
    
    match msg {
    	winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;
            
            webview_set_rect(sc.as_inner_mut().as_inner_mut().oleptr, common::window_rect(hwnd));

            if let Some(ref mut cb) = sc.base_mut().handler_resize {
                let mut sc2: &mut WebView = mem::transmute(ww);
                (cb.as_mut())(sc2, width, height);
            }
            return 0;
        },
        _ => winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

impl_all_defaults!(WebView);
