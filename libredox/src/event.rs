use core::char;
use core::ptr;

use string::*;
use vec::Vec;

use syscall::{sys_alloc, sys_unalloc, sys_trigger};

/// An optional event
pub enum EventOption {
    /// A mouse event
    Mouse(MouseEvent),
    /// A key event
    Key(KeyEvent),
    /// A redraw event
    Redraw(RedrawEvent),
    /// A open event
    Open(OpenEvent),
    /// An unknown event
    Unknown(Event),
    /// No event
    None,
}

/// An event
#[derive(Copy, Clone)]
#[repr(packed)]
pub struct Event {
    pub code: char,
    pub a: isize,
    pub b: isize,
    pub c: isize,
    pub d: isize,
    pub e: isize,
}

impl Event {
    /// Create a null event
    pub fn new() -> Event {
        Event {
            code: '\0',
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
        }
    }

    /// Convert the event ot an optional event
    // TODO: Consider doing this via a From trait.
    pub fn to_option(self) -> EventOption {
        match self.code {
            'm' => EventOption::Mouse(MouseEvent::from_event(self)),
            'k' => EventOption::Key(KeyEvent::from_event(self)),
            'r' => EventOption::Redraw(RedrawEvent::from_event(self)),
            'o' => EventOption::Open(OpenEvent::from_event(self)),
            '\0' => EventOption::None,
            _ => EventOption::Unknown(self),
        }
    }

    /// Event trigger
    pub fn trigger(&self) {
        unsafe {
            let event_ptr: *const Event = self;
            sys_trigger(event_ptr as usize);
        }
    }
}

/// A event related to the mouse
#[derive(Copy, Clone)]
pub struct MouseEvent {
    /// The x coordinate of the mouse
    pub x: isize,
    /// The y coordinate of the mouse
    pub y: isize,
    /// Was the left button pressed?
    pub left_button: bool,
    /// Was the right button pressed?
    pub right_button: bool,
    /// Was the middle button pressed?
    pub middle_button: bool,
}

impl MouseEvent {
    /// Convert to an `Event`
    pub fn to_event(&self) -> Event {
        Event {
            code: 'm',
            a: self.x,
            b: self.y,
            c: self.left_button as isize,
            d: self.middle_button as isize,
            e: self.right_button as isize,
        }
    }

    /// Convert an `Event` to a `MouseEvent`
    pub fn from_event(event: Event) -> MouseEvent {
        MouseEvent {
            x: event.a,
            y: event.b,
            left_button: event.c > 0,
            middle_button: event.d > 0,
            right_button: event.e > 0,
        }
    }

    /// Mouse event trigger
    #[inline]
    pub fn trigger(&self) {
        self.to_event().trigger();
    }
}

/// Escape key
pub const K_ESC: u8 = 0x01;
/// Backspace key
pub const K_BKSP: u8 = 0x0E;
/// Tab key
pub const K_TAB: u8 = 0x0F;
/// Control key
pub const K_CTRL: u8 = 0x1D;
/// Alt key
pub const K_ALT: u8 = 0x38;
/// F1 key
pub const K_F1: u8 = 0x3B;
/// F2 key
pub const K_F2: u8 = 0x3C;
/// F3 key
pub const K_F3: u8 = 0x3D;
/// F4 key
pub const K_F4: u8 = 0x3E;
/// F5 key
pub const K_F5: u8 = 0x3F;
/// F6 key
pub const K_F6: u8 = 0x40;
/// F7 key
pub const K_F7: u8 = 0x41;
/// F8 key
pub const K_F8: u8 = 0x42;
/// F9 key
pub const K_F9: u8 = 0x43;
/// F10 key
pub const K_F10: u8 = 0x44;
/// Home key
pub const K_HOME: u8 = 0x47;
/// Up key
pub const K_UP: u8 = 0x48;
/// Page up key
pub const K_PGUP: u8 = 0x49;
/// Left key
pub const K_LEFT: u8 = 0x4B;
/// Right key
pub const K_RIGHT: u8 = 0x4D;
/// End key
pub const K_END: u8 = 0x4F;
/// Down key
pub const K_DOWN: u8 = 0x50;
/// Page down key
pub const K_PGDN: u8 = 0x51;
/// Delete key
pub const K_DEL: u8 = 0x53;
/// F11 key
pub const K_F11: u8 = 0x57;
/// F12 key
pub const K_F12: u8 = 0x58;

/// A key event (such as a pressed key)
#[derive(Copy, Clone)]
pub struct KeyEvent {
    /// The charecter of the key
    pub character: char,
    /// The scancode of the key
    pub scancode: u8,
    /// Was it pressed?
    pub pressed: bool,
}

impl KeyEvent {
    /// Convert to an `Event`
    pub fn to_event(&self) -> Event {
        Event {
            code: 'k',
            a: self.character as isize,
            b: self.scancode as isize,
            c: self.pressed as isize,
            d: 0,
            e: 0,
        }
    }

    /// Convert from an `Event`
    pub fn from_event(event: Event) -> KeyEvent {
        match char::from_u32(event.a as u32) {
            Option::Some(character) => KeyEvent {
                character: character,
                scancode: event.b as u8,
                pressed: event.c > 0,
            },
            Option::None => KeyEvent {
                character: '\0',
                scancode: event.b as u8,
                pressed: event.c > 0,
            },
        }
    }

    /// Key event trigger
    #[inline]
    pub fn trigger(&self) {
        self.to_event().trigger();
    }
}

pub const REDRAW_NONE: usize = 0;
pub const REDRAW_CURSOR: usize = 1;
pub const REDRAW_ALL: usize = 2;

/// A redraw event
pub struct RedrawEvent {
    pub redraw: usize,
}

impl RedrawEvent {
    /// Convert to an `Event`
    pub fn to_event(&self) -> Event {
        Event {
            code: 'r',
            a: self.redraw as isize,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
        }
    }

    /// Convert from an `Event`
    pub fn from_event(event: Event) -> RedrawEvent {
        RedrawEvent { redraw: event.a as usize }
    }

    /// Redraw trigger
    #[inline]
    pub fn trigger(&self) {
        self.to_event().trigger();
    }
}

/// A "open event" (such as a IO request)
pub struct OpenEvent {
    /// The URL, see wiki.
    pub url_string: String,
}

impl OpenEvent {
    /// Convert to an `Event`
    pub fn to_event(&self) -> Event {
        unsafe {
            let c_str = sys_alloc(self.url_string.len() + 1) as *mut u8;
            if self.url_string.len() > 0 {
                ptr::copy(self.url_string.as_ptr(), c_str, self.url_string.len());
            }
            ptr::write(c_str.offset(self.url_string.len() as isize), 0);
            Event {
                code: 'o',
                a: c_str as isize,
                b: 0,
                c: 0,
                d: 0,
                e: 0,
            }
        }
    }

    /// Convert from an `Event`
    pub fn from_event(event: Event) -> OpenEvent {
        unsafe {
            let mut utf8: Vec<u8> = Vec::new();
            for i in 0..4096 {
                let b = ptr::read((event.a as *const u8).offset(i));
                if b == 0 {
                    break;
                } else {
                    utf8.push(b);
                }
            }
            sys_unalloc(event.a as usize);

            OpenEvent { url_string: String::from_utf8_unchecked(utf8) }
        }
    }

    /// Event trigger
    pub fn trigger(&self) {
        self.to_event().trigger();
    }
}
