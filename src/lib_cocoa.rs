use super::development as webview_dev;

use plygui_cocoa::common::*;

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        common::register_window_class("PlyguiWebView", BASE_CLASS, |decl| {
            decl.add_method(
                sel!(setFrameSize:),
                set_frame_size as extern "C" fn(&mut Object, Sel, NSSize),
            );
        })
    };
    static ref DELEGATE: common::RefClass = unsafe { register_delegate() };
}

const DEFAULT_PADDING: i32 = 6;
const BASE_CLASS: &str = "WKWebView";

pub type WebView = Member<Control<WebViewCocoa>>;

#[repr(C)]
pub struct WebViewCocoa {
    base: common::CocoaControlBase<WebView>,
}

impl webview_dev::WebViewInner for WebViewCocoa {
    fn new() -> Box<super::WebView> {
        let mut i = Box::new(Member::with_inner(
            Control::with_inner(
                WebViewCocoa {
                    base: common::CocoaControlBase::with_params(*WINDOW_CLASS),
                },
                (),
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        let selfptr = i.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
        unsafe {
            (&mut *i.as_inner_mut().as_inner_mut().base.control).set_ivar(common::IVAR, selfptr);
            let delegate: *mut Object = msg_send!(DELEGATE.0, new);
            (&mut *delegate).set_ivar(common::IVAR, selfptr);
            let () = msg_send![
                i.as_inner_mut().as_inner_mut().base.control,
                setUIDelegate: delegate
            ];
            let rect = NSRect::new(NSPoint::new(0f64, 0f64), NSSize::new(0f64, 0f64));
            let conf: *mut Object = msg_send!(class!(WKWebViewConfiguration), new);
            let () = msg_send![i.as_inner_mut().as_inner_mut().base.control, initWithFrame:rect configuration:conf];
        }
        i
    }
    fn go_to(&mut self, site: &str) {
        unsafe {
            let url = NSString::alloc(cocoa::base::nil).init_str(site);
            let url: cocoa_id = msg_send![class!(NSURL), URLWithString: url];
            let url_request: cocoa_id = msg_send![class!(NSURLRequest), requestWithURL: url];
            let () = msg_send![self.base.control, loadRequest: url_request];
        }
    }
}

impl ControlInner for WebViewCocoa {
    fn on_added_to_container(
        &mut self,
        member: &mut MemberBase,
        control: &mut ControlBase,
        _parent: &controls::Container,
        _x: i32,
        _y: i32,
        pw: u16,
        ph: u16,
    ) {
        self.measure(member, control, pw, ph);
        self.base.invalidate();
    }
    fn on_removed_from_container(
        &mut self,
        _: &mut MemberBase,
        _: &mut ControlBase,
        _: &controls::Container,
    ) {
        unsafe {
            self.base.on_removed_from_container();
        }
    }

    fn parent(&self) -> Option<&controls::Member> {
        self.base.parent()
    }
    fn parent_mut(&mut self) -> Option<&mut controls::Member> {
        self.base.parent_mut()
    }
    fn root(&self) -> Option<&controls::Member> {
        self.base.root()
    }
    fn root_mut(&mut self) -> Option<&mut controls::Member> {
        self.base.root_mut()
    }

    #[cfg(feature = "markup")]
    fn fill_from_markup(
        &mut self,
        base: &mut MemberBase,
        control: &mut ControlBase,
        markup: &plygui_api::markup::Markup,
        registry: &mut plygui_api::markup::MarkupRegistry,
    ) {
        fill_from_markup_base!(self, base, markup, registry, WebView, ["WebView"]);
        //TODO webview source
    }
}

impl MemberInner for WebViewCocoa {
    type Id = common::CocoaId;

    fn size(&self) -> (u16, u16) {
        self.base.size()
    }

    fn on_set_visibility(&mut self, base: &mut MemberBase) {
        self.base.on_set_visibility(base);
    }

    unsafe fn native_id(&self) -> Self::Id {
        self.base.control.into()
    }
}

impl HasLayoutInner for WebViewCocoa {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl Drawable for WebViewCocoa {
    fn draw(
        &mut self,
        member: &mut MemberBase,
        _control: &mut ControlBase,
        coords: Option<(i32, i32)>,
    ) {
        if coords.is_some() {
            self.base.coords = coords;
        }
        if let Some((x, y)) = self.base.coords {
            let (_, ph) = self
                .parent_mut()
                .unwrap()
                .is_container_mut()
                .unwrap()
                .size();
            unsafe {
                let mut frame: NSRect = self.base.frame();
                frame.size = NSSize::new(
                    (self.base.measured_size.0 as i32) as f64,
                    (self.base.measured_size.1 as i32) as f64,
                );
                frame.origin = NSPoint::new(
                    x as f64,
                    (ph as i32 - y - self.base.measured_size.1 as i32) as f64,
                );
                let () = msg_send![self.base.control, setFrame: frame];
            }
            if let Some(ref mut cb) = member.handler_resize {
                unsafe {
                    let object: &Object = mem::transmute(self.base.control);
                    let saved: *mut c_void = *object.get_ivar(common::IVAR);
                    let mut ll2: &mut WebView = mem::transmute(saved);
                    (cb.as_mut())(ll2, self.base.measured_size.0, self.base.measured_size.1);
                }
            }
        }
    }
    fn measure(
        &mut self,
        member: &mut MemberBase,
        control: &mut ControlBase,
        parent_width: u16,
        parent_height: u16,
    ) -> (u16, u16, bool) {
        use std::cmp::max;

        let old_size = self.base.measured_size;
        self.base.measured_size = match member.visibility {
            types::Visibility::Gone => (0, 0),
            _ => unsafe {
                let mut label_size = (0, 0);
                let w = match control.layout.width {
                    layout::Size::MatchParent => parent_width as i32,
                    layout::Size::Exact(w) => w as i32,
                    layout::Size::WrapContent => {
                        label_size = common::measure_nsstring(msg_send![self.base.control, title]);
                        label_size.0 as i32 + DEFAULT_PADDING + DEFAULT_PADDING
                    }
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => parent_height as i32,
                    layout::Size::Exact(h) => h as i32,
                    layout::Size::WrapContent => {
                        if label_size.1 < 1 {
                            label_size =
                                common::measure_nsstring(msg_send![self.base.control, title]);
                        }
                        label_size.1 as i32 + DEFAULT_PADDING + DEFAULT_PADDING
                    }
                };
                (max(0, w) as u16, max(0, h) as u16)
            },
        };
        (
            self.base.measured_size.0,
            self.base.measured_size.1,
            self.base.measured_size != old_size,
        )
    }
    fn invalidate(&mut self, _: &mut MemberBase, _: &mut ControlBase) {
        self.base.invalidate();
    }
}

unsafe fn register_delegate() -> common::RefClass {
    let superclass = Class::get("NSObject").unwrap();
    let mut decl = ClassDecl::new("PlyguiWebViewDelegate", superclass).unwrap();

    decl.add_method(
        sel!(viewDidLoad:),
        view_loaded as extern "C" fn(&mut Object, Sel, cocoa_id),
    );
    decl.add_ivar::<*mut c_void>(common::IVAR);

    common::RefClass(decl.register())
}
#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
    use NewWebView;
    WebView::new().into_control()
}
extern "C" fn view_loaded(this: &mut Object, _: Sel, param: cocoa_id) {
    unsafe {
        println!("loaded");
    }
}
extern "C" fn set_frame_size(this: &mut Object, _: Sel, param: NSSize) {
    unsafe {
        let sp = common::member_from_cocoa_id_mut::<WebView>(this).unwrap();
        let () = msg_send![
            super(
                sp.as_inner_mut().as_inner_mut().base.control,
                Class::get(BASE_CLASS).unwrap()
            ),
            setFrameSize: param
        ];
        sp.call_on_resize(param.width as u16, param.height as u16)
    }
}
impl_all_defaults!(WebView);
