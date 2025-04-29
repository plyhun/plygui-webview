use crate::sdk::*;

use plygui_gtk::common::*;

use plygui_gtk::gtk::Widget;
use plygui_gtk::common::{Cast, WidgetExt};
use webkit2gtk::{WebView as GtkWebViewSys, WebViewExt};

use std::str;

pub type WebView = AMember<AControl<AWebView<GtkWebView>>>;

#[repr(C)]
pub struct GtkWebView {
    base: GtkControlBase<WebView>,
}

impl<O: crate::WebView> NewWebViewInner<O> for GtkWebView {
    fn with_uninit(u: &mut mem::MaybeUninit<O>) -> Self {
        let mut sc = Self {
            base: GtkControlBase::with_gtk_widget(GtkWebViewSys::new().upcast::<Widget>()),
        };
        {
            let ptr = u as *mut _ as *mut c_void;
            sc.base.set_pointer(ptr);
        }
        Object::from(sc.base.widget.clone()).downcast::<Widget>().unwrap().connect_size_allocate(on_size_allocate::<O>);
        sc
    }
}
impl WebViewInner for GtkWebView {
    fn new() -> Box<dyn crate::WebView> {        
        let mut b: Box<mem::MaybeUninit<WebView>> = Box::new_uninit();
        let ab = AMember::with_inner(
            AControl::with_inner(
                AWebView::with_inner(
                    <Self as NewWebViewInner<WebView>>::with_uninit(b.as_mut()),
                )
            ),
        );
        unsafe {
	        b.as_mut_ptr().write(ab);
	        b.assume_init()
        }
    }
    fn set_url(&mut self, member: &mut MemberBase, control: &mut ControlBase, url: &str) {
        let ll: Object = Object::from(self.base.widget.clone());
        let ll = ll.downcast::<GtkWebViewSys>().unwrap();
        ll.load_uri(url);
    }
    fn url(&self) -> ::std::borrow::Cow<str> {
        Cow::Owned(self.base.widget().downcast::<GtkWebViewSys>().unwrap().uri().unwrap().into())
    }
}

impl HasLayoutInner for GtkWebView {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl ControlInner for GtkWebView {
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

impl HasNativeIdInner for GtkWebView {
    type Id = GtkWidget;

    fn native_id(&self) -> Self::Id {
        self.base.widget.clone().into()
    }
}

impl HasSizeInner for GtkWebView {
    fn on_size_set(&mut self, _: &mut MemberBase, (width, height): (u16, u16)) -> bool {
        self.base.widget().set_size_request(width as i32, height as i32);
        true
    }
}

impl HasVisibilityInner for GtkWebView {
    fn on_visibility_set(&mut self, _: &mut MemberBase, _: types::Visibility) -> bool {
        self.base.invalidate()
    }
}

impl MemberInner for GtkWebView {}

impl Drawable for GtkWebView {
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
impl Spawnable for GtkWebView {
    fn spawn() -> Box<dyn controls::Control> {
        Self::new().into_control()
    }
}
fn on_size_allocate<O: crate::WebView>(this: &::plygui_gtk::gtk::Widget, _allo: &::plygui_gtk::gtk::Rectangle) {
    use plygui_api::controls::HasSize;

    let mut ll = this.clone().upcast::<Widget>();
    let ll = cast_gtk_widget_to_member_mut::<WebView>(&mut ll).unwrap();

    let measured_size = ll.size();
    ll.call_on_size::<O>(measured_size.0 as u16, measured_size.1 as u16);
}
