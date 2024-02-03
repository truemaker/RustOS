#![no_std]
#![no_main]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rustos::memory::BootInfoFrameAllocator;
use rustos::task::executor::Executor;
use rustos::task::{keyboard,Task};
use rustos::{allocator, gdt, interrupts, memory, println};
use x86_64::{self, VirtAddr};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    x86_64::instructions::interrupts::disable();
    loop {
        x86_64::instructions::hlt();
    }
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    println!("Initialized GDT and IDT. Enabled Interrupts...");

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut page_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    println!("'mapper' and 'page_allocator' initialized...");

    allocator::init_heap(&mut mapper, &mut page_allocator).expect("heap initialization failed");

    println!("Initialized 'heap'. Dynamic memory available...");

    let mut executor = Executor::new();
    println!("Setup Cooperative multi-tasking...");
    executor.spawn(Task::new(keyboard::print_keypresses()));
    println!("Added tasks. Running...");
    executor.run();
}
