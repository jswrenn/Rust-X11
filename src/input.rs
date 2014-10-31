pub use self::modkey::ModkeySet;
pub use self::pointer_mode::PointerMode;
pub use self::keyboard_mode::KeyboardMode;
pub use self::pointer_event_mode::PointerEventMode;
use xcb;

pub mod modkey {
    use xcb;
    pub type ModkeyInt = u16;
    bitflags!{
        #[deriving(Show)] flags ModkeySet: ModkeyInt {
            const SHIFT     = xcb::XCB_MOD_MASK_SHIFT as ModkeyInt,
            const LOCK      = xcb::XCB_MOD_MASK_LOCK as ModkeyInt,
            const CONTROL   = xcb::XCB_MOD_MASK_CONTROL as ModkeyInt,
            const MOD_1     = xcb::XCB_MOD_MASK_1 as ModkeyInt,
            const MOD_2     = xcb::XCB_MOD_MASK_2 as ModkeyInt,
            const MOD_3     = xcb::XCB_MOD_MASK_3 as ModkeyInt,
            const MOD_4     = xcb::XCB_MOD_MASK_4 as ModkeyInt,
            const MOD_5     = xcb::XCB_MOD_MASK_5 as ModkeyInt,
            const ANY       = xcb::XCB_MOD_MASK_ANY as ModkeyInt
        }
}

}

pub type KeycodeInt = xcb::xcb_keycode_t;
new_type!{
#[deriving(Show)]
type Keycode = KeycodeInt
}

///Represents the simultaneous keypress (chord) of a regular key and a number of modifier keys.
pub struct KeyWithModkeySet {
    pub keycode: Keycode,
    pub modkey_set: ModkeySet
}

pub mod pointer_mode {
    use xcb;
    pub type PointerModeInt = u8;
    #[deriving(Show)]
    #[repr(u8)]
    pub enum PointerMode {
        Sync  = xcb::XCB_GRAB_MODE_SYNC as u8,
        Async = xcb::XCB_GRAB_MODE_ASYNC as u8
    }
}

pub mod keyboard_mode {
    use xcb;
    pub type KeyboardModeInt = u8;
    #[deriving(Show)]
    #[repr(u8)]
    pub enum KeyboardMode {
        Sync  = xcb::XCB_GRAB_MODE_SYNC as u8,
        Async = xcb::XCB_GRAB_MODE_ASYNC as u8
    }
}

pub mod pointer_event_mode {
    pub type PointerEventModeInt = u8;
    #[deriving(Show)]
    #[repr(u8)]
    pub enum PointerEventMode {
        SendOnlyToGrabbed = 0,
        SendAlsoToPointed = 1
    }
}
