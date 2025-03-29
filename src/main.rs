#![no_std]
#![no_main]

extern crate alloc;
extern crate panic_halt;
extern crate riscv;

use embedded_alloc::LlffHeap as Heap;
use tinysys_sys::*;

use core::arch::asm;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    {
        #![allow(static_mut_refs)]
        use core::mem::MaybeUninit;

        const HEAP_SIZE: usize = 4 * 1024 * 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }
    dbg!(HEAP.free() / 1024);

    unsafe {
        let mut video_context = EVideoContext {
            m_vmode: EVideoMode_EVM_320_Wide,
            m_cmode: EColorMode_ECM_8bit_Indexed,
            ..Default::default()
        };
        VPUSetVMode(&mut video_context, EVideoScanoutEnable_EVS_Enable);

        let framebuffer = VPUAllocateBuffer((320 * 240) as u32);
        let framebuffer_mem = core::slice::from_raw_parts_mut(framebuffer, 320 * 240);

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
