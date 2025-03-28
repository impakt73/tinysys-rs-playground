#![no_std]
#![no_main]
#![allow(static_mut_refs)]

extern crate panic_halt;

use alloc::{alloc::alloc, format};
use riscv as _;

extern crate alloc;

use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

use core::ptr;

use tinysys_sys::*;

#[unsafe(no_mangle)]
pub extern "C" fn malloc(size: usize) -> *mut u8 {
    unsafe { alloc(alloc::alloc::Layout::from_size_align_unchecked(size, 8)) }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 4 * 1024 * 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    let mut count = 0;
    unsafe {
        let mut video_context: EVideoContext = EVideoContext {
            m_vmode: 0,
            m_cmode: 0,
            m_scanEnable: 0,
            m_strideInWords: 0,
            m_scanoutAddressCacheAligned: 0,
            m_cpuWriteAddressCacheAligned: 0,
            m_graphicsWidth: 0,
            m_graphicsHeight: 0,
            m_consoleWidth: 0,
            m_consoleHeight: 0,
            m_cursorX: 0,
            m_cursorY: 0,
            m_consoleUpdated: 0,
            m_caretX: 0,
            m_caretY: 0,
            m_consoleColor: 0,
            m_caretBlink: 0,
        };
        video_context.m_vmode = EVideoMode_EVM_640_Wide;
        video_context.m_cmode = EColorMode_ECM_16bit_RGB;
        VPUSetVMode(&mut video_context, EVideoScanoutEnable_EVS_Enable);

        let framebuffer_a = VPUAllocateBuffer((640 * 480 * 4) as u32);
        let framebuffer_b = VPUAllocateBuffer((640 * 480 * 4) as u32);

        let mut swap_context: EVideoSwapContext = EVideoSwapContext {
            cycle: 0,
            readpage: ptr::null_mut(),
            writepage: ptr::null_mut(),
            framebufferA: framebuffer_a,
            framebufferB: framebuffer_b,
        };
        VPUSwapPages(&mut video_context, &mut swap_context);

        loop {
            VPUClear(&mut video_context, count);

            VPUWaitVSync();
            VPUSwapPages(&mut video_context, &mut swap_context);

            VPUConsoleClear(&mut video_context);
            let output = format!("Count: {}\n", count);
            VPUConsolePrint(&mut video_context, output.as_ptr(), output.len() as i32);

            count += 1;

            LEDSetState(count);
        }
    }
}
