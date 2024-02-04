use core::cmp::{max, min};

use lazy_static::lazy_static;
use ps2_mouse::{Mouse, MouseState};
use x86_64::{instructions::port::PortReadOnly, structures::idt::InterruptStackFrame};
use spin;

use crate::{interrupts::{InterruptIndex, PICS}, println};

lazy_static! {
    pub static ref MOUSE: spin::Mutex<Mouse> = spin::Mutex::new(Mouse::new());
    pub static ref MOUSE_STATE: spin::Mutex<MouseHandle> = spin::Mutex::new(MouseHandle::new(2));
}

pub fn init_mouse() {
    MOUSE.lock().init().unwrap();
    MOUSE.lock().set_on_complete(on_complete);
}

fn on_complete(mouse_state: MouseState) {
    MOUSE_STATE.lock().mutate(mouse_state, true);
    println!("{:?}", MOUSE_STATE.lock());
}

pub extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = PortReadOnly::new(0x60);
    let packet = unsafe { port.read() };
    MOUSE.lock().process_packet(packet);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Mouse.as_u8());
    }
}

#[derive(Debug)]
pub struct MouseHandle {
    x: i16,
    y: i16,
    mouse_left: bool,
    mouse_right: bool,
    mouse_middle: bool,
    insensitivity: i16,
}

impl MouseHandle {
    pub fn new(insensitivity: i16) -> MouseHandle {
        MouseHandle {
            x: 0,
            y: 0,
            mouse_left: false,
            mouse_middle: false,
            mouse_right: false,
            insensitivity,
        }
    }
    pub fn mutate(&mut self, mouse_state: MouseState, limit: bool) {
        self.x += mouse_state.get_x() / self.insensitivity;
        self.y -= mouse_state.get_y() / self.insensitivity;
        self.mouse_right = mouse_state.right_button_down();
        self.mouse_left = mouse_state.left_button_down();
        if limit {
            self.limit();
        }
    }
    pub fn limit(&mut self) {
        self.x = min(max(self.x,0),80);
        self.y = min(max(self.y,0),25)
    }
}
