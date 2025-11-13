use crate::sdk::*;

use plygui_qt::common::*;
use scintilla_sys::*;

pub type Scintilla = AMember<AControl<AScintilla<QtScintilla>>>;

#[repr(C)]
pub struct QtScintilla {
    base: QtControlBase<Scintilla, QScintillaEditBase>,
    h_command: (bool, QBox<SlotOfSCNotification>),
}

impl<O: crate::Scintilla> NewScintillaInner<O> for QtScintilla {
    fn with_uninit(u: &mut mem::MaybeUninit<O>) -> Self {
        let sc = Self {
            base: QtControlBase::with_params( QScintillaEditBase::new(), event_handler::<O>),
            h_command: (false, unsafe { SlotOfSCNotification::new(NullPtr, move |_| {}) }),
        };
        unsafe {
            let ptr = u as *const _ as u64;
            let qo: &QObject = &sc.base.widget.static_upcast();
            qo.set_property(PROPERTY.as_ptr() as *const i8, &QVariant::from_u64(ptr));
            sc.base.widget.notify().connect(&sc.h_command.1);
        }
        sc
    }
}
impl ScintillaInner for QtScintilla {
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
        unsafe {
            let _ = self.base.widget.send(SCI_SETMARGINWIDTHN as u32, index, width);
        }
    }
    fn set_readonly(&mut self, readonly: bool) {
        unsafe {
            let _ = self.base.widget.send(SCI_SETREADONLY as u32, if readonly { 1 } else { 0 }, 0);
        }
    }
    fn is_readonly(&self) -> bool {
        unsafe { self.base.widget.send(SCI_GETREADONLY, 0, 0) as usize == 1 }
    }
    fn set_codepage(&mut self, cp: crate::Codepage) {
        unsafe {
            let _ = self.base.widget.send(SCI_SETCODEPAGE, cp as usize, 0);
        }
    }
    fn codepage(&self) -> crate::Codepage {
        unsafe { (self.base.widget.send(SCI_GETCODEPAGE, 0, 0) as isize).into() }
    }
    fn append_text(&mut self, text: &str) {
        self.set_codepage(crate::Codepage::Utf8);
        let len = text.len();
        let tptr = text.as_bytes().as_ptr();
        unsafe {
            self.base.widget.send(SCI_APPENDTEXT, len, tptr as isize);
        }
    }
}

impl HasLayoutInner for QtScintilla {
    fn on_layout_changed(&mut self, _base: &mut MemberBase) {
        self.base.invalidate();
    }
}
impl Spawnable for QtScintilla {
    fn spawn() -> Box<dyn controls::Control> {
        Self::new().into_control()
    }
}
impl ControlInner for QtScintilla {
    fn parent(&self) -> Option<&dyn controls::Member> {
        self.base.parent()
    }
    fn parent_mut(&mut self) -> Option<&mut dyn controls::Member> {
        self.base.parent_mut()
    }
    fn root(&self) -> Option<&dyn controls::Member> {
        self.base.root()
    }
    fn root_mut(&mut self) -> Option<&mut dyn controls::Member> {
        self.base.root_mut()
    }
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, _parent: &dyn controls::Container, x: i32, y: i32, pw: u16, ph: u16) {
        control.coords = Some((x, y));
        self.measure(member, control, pw, ph);
        self.base.dirty = false;
        self.draw(member, control);
    }
    fn on_removed_from_container(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, _: &dyn controls::Container) {}
}

impl HasNativeIdInner for QtScintilla {
    type Id = QtId;

    fn native_id(&self) -> Self::Id {
        QtId::from(unsafe { self.base.widget.static_upcast::<QObject>().as_raw_ptr() } as *mut QObject)
    }
}
impl HasVisibilityInner for QtScintilla {
    fn on_visibility_set(&mut self, _: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.set_visibility(value);
        self.base.invalidate()
    }
}
impl HasSizeInner for QtScintilla {
    fn on_size_set(&mut self, _: &mut MemberBase, (width, height): (u16, u16)) -> bool {
        unsafe { self.base.widget.set_fixed_size_2a(width as i32, height as i32); }
        true
    }
}
impl MemberInner for QtScintilla {}

impl Drawable for QtScintilla {
    fn draw(&mut self, member: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(member, control);
    }
    fn measure(&mut self, _member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        let old_size = control.measured;
        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let w = match control.layout.width {
                    layout::Size::MatchParent => parent_width as i32,
                    layout::Size::Exact(w) => w as i32,
                    layout::Size::WrapContent => 42, // TODO min size
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => parent_height as i32,
                    layout::Size::Exact(h) => h as i32,
                    layout::Size::WrapContent => 42, // TODO min size
                };
                (cmp::max(0, w) as u16, cmp::max(0, h) as u16)
            }
        };
        self.base.dirty = control.measured != old_size;
        (control.measured.0, control.measured.1, self.base.dirty)
    }
    fn invalidate(&mut self, _member: &mut MemberBase, _control: &mut ControlBase) {
        self.base.invalidate();
    }
}

fn event_handler<O: crate::Scintilla>(object: &mut QObject, event: &mut QEvent) -> bool {
    match unsafe { event.type_() } {
        QEventType::Resize => {
            if let Some(this) = cast_qobject_to_uimember_mut::<Scintilla>(object) {
                let size = unsafe { 
                    let size = Ptr::from_raw(event).static_downcast::<QResizeEvent>();
                    let size = (
                    	utils::coord_to_size(size.size().width()), 
                    	utils::coord_to_size(size.size().height())
                    );
                    this.inner_mut().base.measured = size;
                    if let layout::Size::WrapContent = this.inner_mut().base.layout.width {
                        this.inner_mut().inner_mut().inner_mut().base.widget.set_minimum_width(size.0 as i32);  
                        this.inner_mut().inner_mut().inner_mut().base.widget.set_maximum_width(size.0 as i32); 
                    }
                    if let layout::Size::WrapContent = this.inner_mut().base.layout.height {
                        this.inner_mut().inner_mut().inner_mut().base.widget.set_minimum_height(size.1 as i32); 
                        this.inner_mut().inner_mut().inner_mut().base.widget.set_maximum_height(size.1 as i32); 
                    }
                    size
                };
                this.call_on_size::<O>(size.0, size.1);
            }
        }
        _ => {}
    }
    false
}
