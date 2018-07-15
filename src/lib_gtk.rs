use super::development as webview_dev;

use plygui_api::{layout, types, controls};
use plygui_api::development::*;	

use plygui_gtk::common;

use gtk::{Cast, Widget, WidgetExt};
use webkit2gtk::{WebView as GtkWebViewSys, WebViewExt};

use std::cmp::max;

pub type WebView = Member<Control<GtkWebView>>;

#[repr(C)]
pub struct GtkWebView {
    base: common::GtkControlBase<WebView>,
}

impl webview_dev::WebViewInner for GtkWebView {
    fn new() -> Box<super::WebView> {
        let mut i = Box::new(Member::with_inner(Control::with_inner(GtkWebView {
                base: common::GtkControlBase::with_gtk_widget(GtkWebViewSys::new().upcast::<Widget>()),
            }, ()), MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut)));
        
        i.as_inner_mut().as_inner_mut().base.widget.connect_size_allocate(on_size_allocate);
        {
        	let ptr = i.as_ref() as *const _ as *mut ::std::os::raw::c_void;
        	i.as_inner_mut().as_inner_mut().base.set_pointer(ptr);
        }
        i
    }
	fn go_to(&mut self, site: &str) {
	    let ll: Widget = self.base.widget.clone().into();
	    let ll = ll.downcast::<GtkWebViewSys>().unwrap();
    	ll.load_uri(site);
	}
}

impl HasLayoutInner for GtkWebView {
	fn on_layout_changed(&mut self, _: &mut MemberBase) {
		//self.apply_padding(unsafe { &mut utils::member_control_base_mut_unchecked(base).control });
		self.base.invalidate();
	}
}

impl ControlInner for GtkWebView {
	fn on_added_to_container(&mut self, base: &mut MemberControlBase, parent: &controls::Container, x: i32, y: i32) {
		let (pw, ph) = parent.draw_area_size();
        self.measure(base, pw, ph);
        println!("{} {} {:?}", ph, ph, self.base.measured_size);
        //self.apply_sized_image(base);
        self.draw(base, Some((x, y)));
	}
    fn on_removed_from_container(&mut self, _: &mut MemberControlBase, _: &controls::Container) {}
    
    fn parent(&self) -> Option<&controls::Member> {
    	self.base.parent().map(|m| m.as_member())
    }
    fn parent_mut(&mut self) -> Option<&mut controls::Member> {
    	self.base.parent_mut().map(|m| m.as_member_mut())
    }
    fn root(&self) -> Option<&controls::Member> {
    	self.base.root().map(|m| m.as_member())
    }
    fn root_mut(&mut self) -> Option<&mut controls::Member> {
    	self.base.root_mut().map(|m| m.as_member_mut())
    }
}

impl MemberInner for GtkWebView {
	type Id = common::GtkWidget;
	
    fn size(&self) -> (u16, u16) {
    	self.base.measured_size
    }
    
    fn on_set_visibility(&mut self, _: &mut MemberBase) {
    	self.base.invalidate()
    }
    
    unsafe fn native_id(&self) -> Self::Id {
    	self.base.widget.clone().into()
    }
}

impl Drawable for GtkWebView {
	fn draw(&mut self, base: &mut MemberControlBase, coords: Option<(i32, i32)>) {
		self.base.draw(base, coords);
	}
    fn measure(&mut self, base: &mut MemberControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
    	let old_size = self.base.measured_size;
    	self.base.measured_size = match base.member.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let w = match base.control.layout.width {
                    layout::Size::MatchParent => parent_width as i32,
                    layout::Size::Exact(w) => w as i32,
                    layout::Size::WrapContent => 42 as i32, //TODO
                };
                let h = match base.control.layout.height {
                    layout::Size::MatchParent => parent_height as i32,
                    layout::Size::Exact(h) => h as i32,
                    layout::Size::WrapContent => 42 as i32, //TODO
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
    fn invalidate(&mut self, _: &mut MemberControlBase) {
    	self.base.invalidate()
    }
}

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
    use NewWebView;
    
	WebView::new().into_control()
}

fn on_size_allocate(this: &::gtk::Widget, _allo: &::gtk::Rectangle) {
    let mut ll1 = this.clone().upcast::<Widget>();
    let mut ll2 = this.clone().upcast::<Widget>();
	let ll1 = common::cast_gtk_widget_to_member_mut::<WebView>(&mut ll1).unwrap();
	let ll2 = common::cast_gtk_widget_to_member_mut::<WebView>(&mut ll2).unwrap();
	
	let measured_size = ll1.as_inner().as_inner().base.measured_size;
	if let Some(ref mut cb) = ll1.base_mut().handler_resize {
        (cb.as_mut())(ll2, measured_size.0 as u16, measured_size.1 as u16);
    }
}

impl_all_defaults!(WebView);
