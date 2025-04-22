use crate::sdk::*;

use plygui_cocoa::common::*;

use std::os::raw::{c_int, c_long, c_ulong, c_void};

lazy_static! {
    static ref WINDOW_CLASS: RefClass = unsafe {
        register_window_class("PlyguiConsole", BASE_CLASS, |decl| {
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
        })
    };
}

pub type Scintilla = AMember<AControl<AScintilla<CocoaScintilla>>>;

const BASE_CLASS: &str = "ScintillaView";

#[repr(C)]
pub struct CocoaScintilla {
    base: CocoaControlBase<Scintilla>,

    fn_ptr: Option<extern "C" fn(*mut c_void, c_int, c_ulong, c_long) -> *mut c_void>,
    self_ptr: Option<*mut c_void>,
}

impl<O: crate::Scintilla> NewScintillaInner<O> for CocoaScintilla {
    fn with_uninit(u: &mut mem::MaybeUninit<O>) -> Self {
        let sc = Self {
            base: CocoaControlBase::with_params(*WINDOW_CLASS, set_frame_size_inner::<O>),
            fn_ptr: None,
            self_ptr: None,
        };
        unsafe {
            let selfptr = u as *mut _ as *mut ::std::os::raw::c_void;
            (&mut *sc.base.control).set_ivar(IVAR, selfptr);
        }
        sc
    }
}
impl ScintillaInner for CocoaScintilla {
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
            (fn_ptr)(self.self_ptr.unwrap(), scintilla_sys::SCI_SETMARGINWIDTHN as i32, index as c_ulong, width as c_long);
        }
    }
    fn set_readonly(&mut self, readonly: bool) {
        if let Some(fn_ptr) = self.fn_ptr {
            (fn_ptr)(self.self_ptr.unwrap(), scintilla_sys::SCI_SETREADONLY as i32, if readonly { 1 } else { 0 }, 0);
        }
    }
    fn is_readonly(&self) -> bool {
        if let Some(fn_ptr) = self.fn_ptr {
            !(fn_ptr)(self.self_ptr.unwrap(), scintilla_sys::SCI_GETREADONLY as i32, 0, 0).is_null()
        } else {
            true
        }
    }
    fn set_codepage(&mut self, cp: crate::Codepage) {
        if let Some(fn_ptr) = self.fn_ptr {
            ((fn_ptr)(self.self_ptr.unwrap(), scintilla_sys::SCI_SETCODEPAGE as i32, cp as c_ulong, 0) as isize);
        }
    }
    fn codepage(&self) -> crate::Codepage {
        if let Some(fn_ptr) = self.fn_ptr {
            ((fn_ptr)(self.self_ptr.unwrap(), scintilla_sys::SCI_GETCODEPAGE as i32, 0, 0) as isize).into()
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

impl ControlInner for CocoaScintilla {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, _parent: &dyn controls::Container, _x: i32, _y: i32, pw: u16, ph: u16) {
        unsafe {
            use scintilla_sys::{SCI_GETDIRECTFUNCTION, SCI_GETDIRECTPOINTER};

            let fn_ptr: extern "C" fn(*mut c_void, c_int, c_ulong, c_long) -> *mut c_void = msg_send![self.base.control, message:SCI_GETDIRECTFUNCTION wParam:0 lParam:0];
            let self_ptr: *mut c_void = msg_send![self.base.control, message:SCI_GETDIRECTPOINTER wParam:0 lParam:0];

            self.fn_ptr = Some(fn_ptr);
            self.self_ptr = Some(self_ptr);
        }
        self.measure(member, control, pw, ph);
    }
    fn on_removed_from_container(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &dyn controls::Container) {
        self.fn_ptr = None;
        self.self_ptr = None;
        unsafe {
            self.base.on_removed_from_container();
        }
    }

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

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, member: &mut MemberBase, control: &mut ControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        fill_from_markup_base!(self, base, markup, registry, Scintilla, ["Scintilla"]);
    }
}

impl HasLayoutInner for CocoaScintilla {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl Drawable for CocoaScintilla {
    fn draw(&mut self, _member: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(control.coords, control.measured);
    }
    fn measure(&mut self, _: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        let old_size = control.measured;
        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let w = match control.layout.width {
                    layout::Size::MatchParent => parent_width,
                    layout::Size::Exact(w) => w,
                    layout::Size::WrapContent => {
                        42 as u16 // TODO min_width
                    }
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => parent_height,
                    layout::Size::Exact(h) => h,
                    layout::Size::WrapContent => {
                        42 as u16 // TODO min_height
                    }
                };
                (w, h)
            }
        };
        (control.measured.0, control.measured.1, control.measured != old_size)
    }
    fn invalidate(&mut self, _: &mut MemberBase, _: &mut ControlBase) {
        self.base.invalidate();
    }
}

impl HasNativeIdInner for CocoaScintilla {
    type Id = CocoaId;

    fn native_id(&self) -> Self::Id {
        self.base.control.into()
    }
}

impl HasSizeInner for CocoaScintilla {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
        self.base.invalidate();
        true
    }
}
impl Spawnable for CocoaScintilla {
    fn spawn() -> Box<dyn controls::Control> {
        Self::new().into_control()
    }
}
impl HasVisibilityInner for CocoaScintilla {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl MemberInner for CocoaScintilla {}
extern "C" fn set_frame_size(this: &mut Object, sel: Sel, param: NSSize) {
    unsafe {
        let b = member_from_cocoa_id_mut::<Scintilla>(this).unwrap();
        let b2 = member_from_cocoa_id_mut::<Scintilla>(this).unwrap();
        (b.inner().inner().inner().base.resize_handler)(b2, sel, param)
    }
}
extern "C" fn set_frame_size_inner<O: crate::Scintilla>(this: &mut Scintilla, _: Sel, param: NSSize) {
    unsafe {
        let () = msg_send![super(this.inner_mut().inner_mut().inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), setFrameSize: param];
        this.call_on_size::<O>(param.width as u16, param.height as u16)
    }
}
