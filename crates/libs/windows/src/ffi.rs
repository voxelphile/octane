pub type HResult = i32;
pub type HString = *mut ();
pub type IUnknown = *mut ();
pub type IInspectable = *mut ();
pub type PStr = *mut i8;
pub type PwStr = *mut i16;
pub type PcStr = *const i8;
pub type PcwStr = *const i16;
pub type LpcStr = *const i8;
pub type Word = u16;
pub type Atom = Word;

pub type LongPtr = i64;
pub type UintPtr = u32;

pub type LResult = LongPtr;

pub type PVoid = *mut ();
pub type Handle = PVoid;
pub type Hwnd = Handle;
pub type WParam = UintPtr;
pub type LParam = LongPtr;
pub type HInstance = Handle;
pub type HIcon = Handle;
pub type HCursor = Handle;
pub type HBrush = Handle;

pub type WndProc = extern "system" fn(Hwnd, u32, WParam, LParam) -> LResult;

#[link(name = "win32", kind = "static")]
extern "system" {
    pub fn RegisterClassA(wnd_class: *const WndClassA) -> Atom; 
}

#[repr(C)]
pub struct WndClassA {
    pub style: u32,
    pub lpfn_wnd_proc: WndProc, 
    pub cb_cls_extra: i32,
    pub cb_wnd_extra: i32,
    pub h_instance: HInstance,
    pub h_icon: HIcon,
    pub h_cursor: HCursor,
    pub hbr_background: HBrush,
    pub lpsz_menu_name: LpcStr,
    pub lpsz_class_name: LpcStr,
}


