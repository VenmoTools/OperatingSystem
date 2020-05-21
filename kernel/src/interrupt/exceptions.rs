use system::interrupt_frame;

use crate::println;

////////////////////// Exceptions /////////////////////////////
interrupt_frame!(divide_by_zero,stack,{
    println!("Divide by zero: {:?}",stack.dump());
});

interrupt_frame!(debug,stack,{
    println!("Debug trap {:?}",stack.dump());
});

interrupt_frame!(non_maskable,stack,{
    println!("Non-maskable interrupt: {:?}",stack.dump());
});

interrupt_frame!(breakpoint,stack,{
    println!("Breakpoint trap: {:?}",stack.dump());
});

interrupt_frame!(invalid_opcode, stack, {
    println!("Invalid opcode fault: {:?}",stack.dump());
});

interrupt_frame!(page_fault, stack, {
    println!("Invalid opcode fault: {:?}",stack.dump());
});

interrupt_frame!(double_fault, stack, {
    println!("Invalid opcode fault: {:?}",stack.dump());
});

interrupt_frame!(invalid_tss,stack,{
    println!("invalid_tss: {:?}",stack.dump());
});
interrupt_frame!(security_exception,stack,{
    println!("security_exception: {:?}",stack.dump());
});
interrupt_frame!(segment_not_present,stack,{
    println!("segment_not_present: {:?}",stack.dump());
});
interrupt_frame!(alignment_check,stack,{
    println!("alignment_check: {:?}",stack.dump());
});
interrupt_frame!(bound_range_exceeded,stack,{
    println!("bound_range_exceeded: {:?}",stack.dump());
});
interrupt_frame!(device_not_available,stack,{
    println!("device_not_available: {:?}",stack.dump());
});
interrupt_frame!(general_protection_fault,stack,{
    println!("general_protection_fault: {:?}",stack.dump());
});
interrupt_frame!(machine_check,stack,{
    println!("machine_check: {:?}",stack.dump());
});
interrupt_frame!(non_maskable_interrupt,stack,{
    println!("non_maskable_interrupt: {:?}",stack.dump());
});
interrupt_frame!(virtualization,stack,{
    println!("virtualization: {:?}",stack.dump());
});
interrupt_frame!(x87_floating_point,stack,{
    println!("x87_floating_point: {:?}",stack.dump());
});
interrupt_frame!(stack_segment_fault,stack,{
    println!("stack_segment_fault: {:?}",stack.dump());
});
interrupt_frame!(simd_floating_point,stack,{
    println!("simd_floating_point: {:?}",stack.dump());
});
interrupt_frame!(overflow,stack,{
    println!("overflow: {:?}",stack.dump());
});