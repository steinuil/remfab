//! Constants from /include/uapi/linux/fb.h
//!
//! https://github.com/torvalds/linux/blob/master/include/uapi/linux/fb.h

pub mod fb_type {
    pub const PACKED_PIXELS: u32 = 0;
    pub const PLANES: u32 = 1;
    pub const INTERLEAVED_PLANES: u32 = 2;
    pub const TEXT: u32 = 3;
    pub const VGA_PLANES: u32 = 4;
    pub const FOURCC: u32 = 5;
}

pub mod fb_aux_text {
    pub const MDA: u32 = 0;
    pub const CGA: u32 = 1;
    pub const S3_MMIO: u32 = 2;
    pub const MGA_STEP16: u32 = 3;
    pub const MGA_STEP8: u32 = 4;
    // pub const SVGA_GROUP: u32 = 8;
    // pub const SVGA_MASK: u32 = 7;
    pub const SVGA_STEP2: u32 = 8;
    pub const SVGA_STEP4: u32 = 9;
    pub const SVGA_STEP8: u32 = 10;
    pub const SVGA_STEP16: u32 = 11;
    // pub const SVGA_LAST: u32 = 15;
}

pub mod fb_aux_vga_planes {

    pub const VGA4: u32 = 0;
    pub const CFB4: u32 = 1;
    pub const CFB8: u32 = 2;
}

pub mod fb_visual {
    pub const MONO01: u32 = 0;
    pub const MONO10: u32 = 1;
    pub const TRUECOLOR: u32 = 2;
    pub const PSEUDOCOLOR: u32 = 3;
    pub const DIRECTCOLOR: u32 = 4;
    pub const STATIC_PSEUDOCOLOR: u32 = 5;
    pub const FOURCC: u32 = 6;
}

pub mod vesa {
    pub const NO_BLANKING: i32 = 0;
    pub const VSYNC_SUSPEND: i32 = 1;
    pub const HSYNC_SUSPEND: i32 = 2;
    pub const POWERDOWN: i32 = VSYNC_SUSPEND | HSYNC_SUSPEND;
}

pub mod ioctl {
    pub const FBIOGET_VSCREENINFO: u16 = 0x4600;
    pub const FBIOPUT_VSCREENINFO: u16 = 0x4601;
    pub const FBIOGET_FSCREENINFO: u16 = 0x4602;
    pub const FBIOPAN_DISPLAY: u16 = 0x4606;
    pub const FBIOBLANK: u16 = 0x4611;
}
