use super::development as webview_dev;

use plygui_cocoa::common::*;

use std::os::raw::c_char;

lazy_static! {
    static ref WINDOW_CLASS: RefClass = unsafe {
        register_window_class("PlyguiWebView", BASE_CLASS, |decl| {
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
        })
    };
    static ref DELEGATE: RefClass = unsafe { register_delegate() };
}

const DEFAULT_PADDING: i32 = 6;
const BASE_CLASS: &str = "WKWebView";

pub type WebView = Member<Control<WebViewCocoa>>;

#[repr(C)]
pub struct WebViewCocoa {
    base: CocoaControlBase<WebView>,
}

impl webview_dev::WebViewInner for WebViewCocoa {
    fn new() -> Box<super::WebView> {
        let mut i = Box::new(Member::with_inner(
            Control::with_inner(
                WebViewCocoa {
                    base: CocoaControlBase::with_params(*WINDOW_CLASS),
                },
                (),
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        let selfptr = i.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
        unsafe {
            (&mut *i.as_inner_mut().as_inner_mut().base.control).set_ivar(IVAR, selfptr);
            let delegate: *mut Object = msg_send!(DELEGATE.0, new);
            (&mut *delegate).set_ivar(IVAR, selfptr);
            let () = msg_send![i.as_inner_mut().as_inner_mut().base.control, setUIDelegate: delegate];
            let rect = NSRect::new(NSPoint::new(0f64, 0f64), NSSize::new(0f64, 0f64));
            let conf: *mut Object = msg_send!(class!(WKWebViewConfiguration), new);
            let () = msg_send![i.as_inner_mut().as_inner_mut().base.control, initWithFrame:rect configuration:conf];
        }
        i
    }
    fn set_url(&mut self, site: &str) {
        unsafe {
            let url = NSString::alloc(nil).init_str(site);
            let url: cocoa_id = msg_send![class!(NSURL), URLWithString: url];
            let url_request: cocoa_id = msg_send![class!(NSURLRequest), requestWithURL: url];
            let () = msg_send![self.base.control, loadRequest: url_request];
        }
    }
    fn url(&self) -> ::std::borrow::Cow<str> {
        unsafe {
            let url: cocoa_id = msg_send![self.base.control, URL];
            let title: cocoa_id = msg_send![url, absoluteString];
            let title: *const c_void = msg_send![title, UTF8String];
            ffi::CStr::from_ptr(title as *const c_char).to_string_lossy()
        }
    }
}

impl ControlInner for WebViewCocoa {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, _parent: &controls::Container, _x: i32, _y: i32, pw: u16, ph: u16) {
        self.measure(member, control, pw, ph);
        self.base.invalidate();
    }
    fn on_removed_from_container(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &controls::Container) {
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
    fn fill_from_markup(&mut self, base: &mut MemberBase, control: &mut ControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        fill_from_markup_base!(self, base, markup, registry, WebView, ["WebView"]);
        //TODO webview source
    }
}

impl HasNativeIdInner for WebViewCocoa {
    type Id = CocoaId;

    unsafe fn native_id(&self) -> Self::Id {
        self.base.control.into()
    }
}

impl HasSizeInner for WebViewCocoa {
    fn on_size_set(&mut self, base: &mut MemberBase, (width, height): (u16, u16)) -> bool {
        use plygui_api::controls::HasLayout;

        let this = base.as_any_mut().downcast_mut::<WebView>().unwrap();
        this.set_layout_width(layout::Size::Exact(width));
        this.set_layout_width(layout::Size::Exact(height));
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for WebViewCocoa {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl MemberInner for WebViewCocoa {}

impl HasLayoutInner for WebViewCocoa {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl Drawable for WebViewCocoa {
    fn draw(&mut self, _member: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(control.coords, control.measured);
    }
    fn measure(&mut self, _member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        use std::cmp::max;

        let old_size = control.measured;
        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => unsafe {
                let mut label_size = (0, 0);
                let w = match control.layout.width {
                    layout::Size::MatchParent => parent_width as i32,
                    layout::Size::Exact(w) => w as i32,
                    layout::Size::WrapContent => {
                        label_size = measure_nsstring(msg_send![self.base.control, title]);
                        label_size.0 as i32 + DEFAULT_PADDING + DEFAULT_PADDING
                    }
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => parent_height as i32,
                    layout::Size::Exact(h) => h as i32,
                    layout::Size::WrapContent => {
                        if label_size.1 < 1 {
                            label_size = measure_nsstring(msg_send![self.base.control, title]);
                        }
                        label_size.1 as i32 + DEFAULT_PADDING + DEFAULT_PADDING
                    }
                };
                (max(0, w) as u16, max(0, h) as u16)
            },
        };
        println!("sized to {:?}", control.measured);
        (control.measured.0, control.measured.1, control.measured != old_size)
    }
    fn invalidate(&mut self, _: &mut MemberBase, _: &mut ControlBase) {
        self.base.invalidate();
    }
}

unsafe fn register_delegate() -> RefClass {
    let superclass = Class::get("NSObject").unwrap();
    let mut decl = ClassDecl::new("PlyguiWebViewDelegate", superclass).unwrap();

    decl.add_method(sel!(viewDidLoad:), view_loaded as extern "C" fn(&mut Object, Sel, cocoa_id));
    decl.add_ivar::<*mut c_void>(IVAR);

    RefClass(decl.register())
}
#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
    use crate::NewWebView;

    WebView::new().into_control()
}
extern "C" fn view_loaded(_this: &mut Object, _: Sel, _param: cocoa_id) {
    println!("loaded");
}
extern "C" fn set_frame_size(this: &mut Object, _: Sel, param: NSSize) {
    unsafe {
        let sp = member_from_cocoa_id_mut::<WebView>(this).unwrap();
        let () = msg_send![super(sp.as_inner_mut().as_inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), setFrameSize: param];
        sp.call_on_size(param.width as u16, param.height as u16)
    }
}
default_impls_as!(WebView);
