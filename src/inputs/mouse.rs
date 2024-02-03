use lazy_static::lazy_static;
use ps2_mouse::{Mouse, MouseState};
use x86_64::{instructions::port::PortReadOnly, structures::idt::InterruptStackFrame};
use spin;

use crate::{interrupts::{InterruptIndex, PICS}, println};

lazy_static! {
    pub static ref MOUSE: spin::Mutex<Mouse> = spin::Mutex::new(Mouse::new());
}

pub fn init_mouse() {
    MOUSE.lock().init().unwrap();
    MOUSE.lock().set_on_complete(on_complete);
}

fn on_complete(mouse_state: MouseState) {
    println!("{:?}", mouse_state);
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
