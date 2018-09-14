use super::development as webview_dev;

use plygui_win32::common::*;

#[repr(C)]
struct OleWebView(u8);

lazy_static! {
    pub static ref WINDOW_CLASS: Vec<u16> = unsafe { register_window_class() };
}

pub type WebView = Member<Control<WebViewWin32>>;
extern "C" {
    fn webview_new_with_parent(parent: windef::HWND) -> *mut OleWebView;
    fn webview_delete(thisptr: *mut OleWebView);
    fn webview_navigate(thisptr: *mut OleWebView, sz_url: *const u16);
    fn webview_set_rect(thisptr: *mut OleWebView, rect: windef::RECT);
}

enum State {
    Attached(*mut OleWebView),
    Unattached(String),
}

#[repr(C)]
pub struct WebViewWin32 {
    base: WindowsControlBase<WebView>,

    state: State,
}

impl Drop for WebViewWin32 {
    fn drop(&mut self) {
        if let State::Attached(oleptr) = self.state {
            unsafe { webview_delete(oleptr) };
            self.state = State::Unattached(String::new());
        }
    }
}

impl webview_dev::WebViewInner for WebViewWin32 {
    fn new() -> Box<super::WebView> {
        let i = Box::new(Member::with_inner(
            Control::with_inner(
                WebViewWin32 {
                    base: WindowsControlBase::new(),
                    state: State::Unattached(String::new()),
                },
                (),
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        i
    }
    fn go_to(&mut self, site: &str) {
        match self.state {
            State::Attached(oleptr) => {
                let mut site = OsStr::new(if site.is_empty() { "about:blank" } else { site })
                    .encode_wide()
                    .chain(Some(0).into_iter())
                    .collect::<Vec<_>>();
                site.push(0);
                unsafe {
                    webview_navigate(oleptr, site.as_ptr());
                }
            }
            State::Unattached(ref mut address) => {
                *address = site.into();
            }
        }
    }
}

impl ControlInner for WebViewWin32 {
    fn on_added_to_container(
        &mut self,
        member: &mut MemberBase,
        control: &mut ControlBase,
        parent: &controls::Container,
        x: i32,
        y: i32,
        pw: u16,
        ph: u16,
    ) {
        let selfptr = member as *mut _ as *mut c_void;
        let (hwnd, id) = unsafe {
            self.base.hwnd = parent.native_id() as windef::HWND; // required for measure, as we don't have own hwnd yet
            let (w, h, _) = self.measure(member, control, pw, ph);
            create_control_hwnd(
                x as i32,
                y as i32,
                w as i32,
                h as i32,
                self.base.hwnd,
                0,
                WINDOW_CLASS.as_ptr(),
                "",
                winuser::WS_TABSTOP | winuser::CS_HREDRAW | winuser::CS_VREDRAW,
                selfptr,
                None,
            )
        };
        self.base.hwnd = hwnd;
        self.base.subclass_id = id;
    }
    fn on_removed_from_container(
        &mut self,
        _member: &mut MemberBase,
        _control: &mut ControlBase,
        _: &controls::Container,
    ) {
        destroy_hwnd(self.base.hwnd, self.base.subclass_id, None);
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
    fn fill_from_markup(
        &mut self,
        base: &mut development::MemberControlBase,
        markup: &plygui_api::markup::Markup,
        registry: &mut plygui_api::markup::MarkupRegistry,
    ) {
        fill_from_markup_base!(self, base, markup, registry, WebView, ["WebView"]);
        //TODO webview source
    }
}

impl HasLayoutInner for WebViewWin32 {
    fn on_layout_changed(&mut self, _base: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl MemberInner for WebViewWin32 {
    type Id = Hwnd;

    fn size(&self) -> (u16, u16) {
        self.base.size()
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
            self.base.invalidate();
        }
    }
    unsafe fn native_id(&self) -> Self::Id {
        self.base.hwnd.into()
    }
}

impl Drawable for WebViewWin32 {
    fn draw(
        &mut self,
        _member: &mut MemberBase,
        _control: &mut ControlBase,
        coords: Option<(i32, i32)>,
    ) {
        self.base.draw(coords);
    }
    fn measure(
        &mut self,
        member: &mut MemberBase,
        control: &mut ControlBase,
        w: u16,
        h: u16,
    ) -> (u16, u16, bool) {
        let old_size = self.base.measured_size;

        self.base.measured_size = match member.visibility {
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
                (cmp::max(0, w as i32) as u16, cmp::max(0, h as i32) as u16)
            }
        };
        (
            self.base.measured_size.0,
            self.base.measured_size.1,
            self.base.measured_size != old_size,
        )
    }
    fn invalidate(&mut self, _member: &mut MemberBase, _control: &mut ControlBase) {
        self.base.invalidate()
    }
}

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
    use super::NewWebView;

    WebView::new().into_control()
}

unsafe fn register_window_class() -> Vec<u16> {
    let class_name = OsStr::new("PlyguiWin32Browser")
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>();
    let class = winuser::WNDCLASSW {
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
    };
    winuser::RegisterClassW(&class);
    class_name
}

unsafe extern "system" fn whandler(
    hwnd: windef::HWND,
    msg: minwindef::UINT,
    wparam: minwindef::WPARAM,
    lparam: minwindef::LPARAM,
) -> minwindef::LRESULT {
    let ww = winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA);
    if ww == 0 {
        if winuser::WM_CREATE == msg {
            use development::WebViewInner;

            let cs: &winuser::CREATESTRUCTW = mem::transmute(lparam);
            winuser::SetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA, cs.lpCreateParams as isize);
            let sc: &mut WebView = mem::transmute(cs.lpCreateParams);
            let address =
                if let State::Unattached(ref address) = sc.as_inner_mut().as_inner_mut().state {
                    address.clone().into()
                } else {
                    String::new()
                };
            sc.as_inner_mut().as_inner_mut().state = State::Attached(webview_new_with_parent(hwnd));
            sc.as_inner_mut().as_inner_mut().go_to(address.as_ref());
        }
        return winuser::DefWindowProcW(hwnd, msg, wparam, lparam);
    }

    let sc: &mut WebView = mem::transmute(ww);

    match msg {
        winuser::WM_SIZE => {
            let width = lparam as u16;
            let height = (lparam >> 16) as u16;

            //TODO proper padding
            if let State::Attached(oleptr) = sc.as_inner_mut().as_inner_mut().state {
                webview_set_rect(oleptr, window_rect(hwnd));
            }

            sc.call_on_resize(width, height);
            return 0;
        }
        _ => winuser::DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

impl_all_defaults!(WebView);
