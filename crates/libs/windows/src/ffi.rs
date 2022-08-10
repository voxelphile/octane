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
pub type DWord = u32;
pub type Atom = Word;
pub type Bool = i32;

pub type LongPtr = *mut i64;
pub type UintPtr = *mut u32;

pub type LResult = LongPtr;

pub type PVoid = *mut ();
pub type LpVoid = PVoid;
pub type Handle = PVoid;
pub type Hwnd = Handle;
pub type WParam = UintPtr;
pub type LParam = LongPtr;
pub type HInstance = Handle;
pub type HIcon = Handle;
pub type HCursor = Handle;
pub type HBrush = Handle;
pub type HMenu = Handle;

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Point(pub i32, pub i32);
#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Rect(pub i32, pub i32, pub i32, pub i32);

pub type WndProc = extern "system" fn(Hwnd, u32, WParam, LParam) -> LResult;


#[allow(overflowing_literals)]
pub const USE_DEFAULT: i32 = 0x80000000;

#[link(name = "User32")]
extern "system" {
    pub fn RegisterClassA(wnd_class: *const WndClassA) -> Atom; 
    pub fn CreateWindowExA(
        ex_style: DWord,
        class_name: LpcStr,
        window_name: LpcStr,
        dw_style: DWord,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: Hwnd,
        menu: HMenu,
        instance: HInstance,
        lp_param: LpVoid,
    ) -> Hwnd;
    pub fn ShowWindow(hwnd: Hwnd, cmd_show: i32);
    pub fn DefWindowProcA(hwnd: Hwnd, u_msg: u32, w_param: WParam, l_param: LParam) -> LResult;
    pub fn GetWindowPlacement(hwnd: Hwnd, wp: *mut WindowPlacement) -> Bool;
    pub fn SetWindowPlacement(hwnd: Hwnd, wp: *const WindowPlacement) -> Bool;
    pub fn GetWindowLongA(hwnd: Hwnd, index: i32) -> i32;
    pub fn SetWindowLongA(hwnd: Hwnd, index: i32, new: i32) -> i32;
    pub fn GetWindowLongPtrA(hwnd: Hwnd, index: i32) -> LongPtr;
    pub fn SetWindowLongPtrA(hwnd: Hwnd, index: i32, new: LongPtr) -> i32;
    pub fn SetWindowTextA(hwnd: Hwnd, text: LpcStr) -> Bool;
    pub fn ShowCursor(show: Bool);
    pub fn GetWindowRect(hwnd: Hwnd, rect: *mut Rect) -> Bool;
    pub fn GetClientRect(hwnd: Hwnd, rect: *mut Rect) -> Bool;
    pub fn SetCursorPos(x: i32, y: i32) -> Bool;
    pub fn PeekMessageA(msg: *mut Msg, hwnd: Hwnd, msg_filter_min: u32, msg_filter_max: u32, remove_msg: u32) -> Bool;
    pub fn TranslateMessage(msg: *const Msg) -> Bool;
    pub fn DispatchMessageW(msg: *const Msg) -> LResult;
    pub fn SetCapture(hwnd: Hwnd) -> Hwnd;
    pub fn ReleaseCapture() -> Bool;
}

#[repr(C)]
pub struct WndClassA {
    pub style: u32,
    pub wnd_proc: WndProc, 
    pub cls_extra: i32,
    pub wnd_extra: i32,
    pub instance: HInstance,
    pub icon: HIcon,
    pub cursor: HCursor,
    pub background: HBrush,
    pub menu_name: LpcStr,
    pub class_name: LpcStr,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct WindowPlacement {
    pub length: u32,
    pub flags: u32,
    pub show_cmd: u32,
    pub min_position: Point,
    pub max_position: Point,
    pub normal_position: Rect,
    pub device: Rect,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Msg {
    hwnd: Hwnd,
    message: u32,
    w_param: WParam,
    l_param: LParam,
    time: u32,
    pt: Point,
    l_private: u32,
}
