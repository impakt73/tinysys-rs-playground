#![no_std]
#![no_main]
#![allow(static_mut_refs)]

extern crate panic_halt;

use alloc::{alloc::alloc, slice};
use riscv as _;

extern crate alloc;

use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

use core::arch::asm;

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
        video_context.m_vmode = EVideoMode_EVM_320_Wide;
        video_context.m_cmode = EColorMode_ECM_8bit_Indexed;
        VPUSetVMode(&mut video_context, EVideoScanoutEnable_EVS_Enable);

        let framebuffer = VPUAllocateBuffer((320 * 240) as u32);
        let framebuffer_mem = slice::from_raw_parts_mut(framebuffer, 320 * 240);

        VPUSetWriteAddress(&mut video_context, framebuffer as u32);
        VPUSetScanoutAddress(&mut video_context, framebuffer as u32);
        VPUClear(&mut video_context, 0x03030303);

        for i in 0..=255 {
            let x: u32 = i & 15;
            let y: u32 = (i >> 4) & 3;
            let z: u32 = x << (y << 2);
            let r = z & 0xff;
            let g = (z >> 4) & 0xff;
            let b = (z >> 8) & 0xff;
            VPUSetPal(i as u8, r, g, b);
        }

        let mut count = 0;
        loop {
            let pixel_index = count % (320 * 240);
            let color = count % 255;

            framebuffer_mem[pixel_index] = color as u8;

            // Flush CPU Data Cache
            asm!(".word 0xFC000073");

            count += 1;
        }
    }
}
