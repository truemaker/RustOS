use crate::gdt;
use crate::inputs::mouse::mouse_interrupt_handler;
use crate::println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX); // new
        }
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(kb_handler);
        idt[InterruptIndex::Mouse.as_usize()]
            .set_handler_fn(mouse_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
    unsafe {
        let mut masks = PICS.lock().read_masks();
        masks[0] = 0xff ^ (PICMasks::Timer.as_u8() | PICMasks::Keyboard.as_u8() | PICMasks::Cascade.as_u8());
        masks[1] = 0xff ^ (PICMasksB::Mouse.as_u8());
        PICS.lock().write_masks(masks[0], masks[1]);
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    //print!(".");
    crate::task::timer::timer_tick();
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn kb_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::inputs::keyboard::add_scancode(scancode); // new

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

// PICs

use pic8259::ChainedPics;
use spin;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PICMasks {
    Timer = 1,
    Keyboard = 2,
    Cascade = 4,
    COM2 = 8,
    COM1 = 16,
    LPT2 = 32,
    Floppy = 64,
    LPT1 = 128,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PICMasksB {
    CMOS = 1,
    LegacySCSI = 2,
    SCSI0 = 4,
    SCSI1 = 8,
    Mouse = 16,
}

impl PICMasks {
    fn as_u8(self) -> u8 {
        self as u8
    }
}

impl PICMasksB {
    fn as_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Mouse = PIC_2_OFFSET + 4,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}
