use super::development as webview_dev;

use plygui_api::development::*;
use plygui_api::{controls, layout, types};

use plygui_gtk::common::*;

use gtk::{Cast, Widget, WidgetExt};
use webkit2gtk::{WebView as GtkWebViewSys, WebViewExt};

use std::cmp::max;

pub type WebView = AMember<AControl<GtkWebView>>;

#[repr(C)]
pub struct GtkWebView {
    base: GtkControlBase<WebView>,
}

impl webview_dev::WebViewInner for GtkWebView {
    fn new() -> Box<super::WebView> {
        let mut i = Box::new(Member::with_inner(
            Control::with_inner(
                GtkWebView {
                    base: GtkControlBase::with_gtk_widget(GtkWebViewSys::new().upcast::<Widget>()),
                },
                (),
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));

        Object::from(i.as_inner_mut().as_inner_mut().base.widget.clone()).downcast::<Widget>().unwrap().connect_size_allocate(on_size_allocate);
        {
            let ptr = i.as_ref() as *const _ as *mut ::std::os::raw::c_void;
            i.as_inner_mut().as_inner_mut().base.set_pointer(ptr);
        }
        i
    }
    fn set_url(&mut self, site: &str) {
        let ll: Object = Object::from(self.base.widget.clone());
        let ll = ll.downcast::<GtkWebViewSys>().unwrap();
        ll.load_uri(site);
    }
    fn url(&self) -> ::std::borrow::Cow<str> {
        let ll: Object = Object::from(self.base.widget.clone());
        let ll = ll.downcast::<GtkWebViewSys>().unwrap();
        Cow::Owned(ll.get_uri().unwrap_or(String::new()))
    }
}

impl HasLayoutInner for GtkWebView {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        //self.apply_padding(unsafe { &mut utils::member_control_base_mut_unchecked(base).control });
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
    fn root(&self) -> Option<&controls::Member> {
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
    fn measure(&mut self, _member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        let old_size = control.measured;
        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let w = match control.layout.width {
                    layout::Size::MatchParent => parent_width as i32,
                    layout::Size::Exact(w) => w as i32,
                    layout::Size::WrapContent => 42 as i32, //TODO
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => parent_height as i32,
                    layout::Size::Exact(h) => h as i32,
                    layout::Size::WrapContent => 42 as i32, //TODO
                };
                (max(0, w) as u16, max(0, h) as u16)
            }
        };
        (control.measured.0, control.measured.1, control.measured != old_size)
    }
    fn invalidate(&mut self, _: &mut MemberBase, _: &mut ControlBase) {
        self.base.invalidate();
    }
}

impl Spawnable for GtkButton {
    fn spawn() -> Box<dyn controls::Control> {
        Self::new().into_control()
    }
}

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<dyn controls::Control> {
    use crate::NewWebView;

    WebView::new().into_control()
}

fn on_size_allocate(this: &::gtk::Widget, _allo: &::gtk::Rectangle) {
    let mut ll = this.clone().upcast::<Widget>();
    let ll = cast_gtk_widget_to_member_mut::<WebView>(&mut ll).unwrap();

    let measured_size = ll.as_inner().base().measured;
    ll.call_on_size(measured_size.0 as u16, measured_size.1 as u16);
}

default_impls_as!(WebView);
