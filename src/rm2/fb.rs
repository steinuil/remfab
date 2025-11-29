use std::{ffi::c_ulong, mem::MaybeUninit, num::NonZeroU16};

use nix::{errno::Errno, libc::c_int};

mod raw {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextType {
    MDA,
    CGA,
    S3MMIO,
    MGAStep16,
    MGAStep8,
    SVGAStep2,
    SVGAStep4,
    SVGAStep8,
    SVGAStep16,
}

impl TryFrom<u32> for TextType {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            raw::fb_aux_text::MDA => Ok(TextType::MDA),
            raw::fb_aux_text::CGA => Ok(TextType::CGA),
            raw::fb_aux_text::S3_MMIO => Ok(TextType::S3MMIO),
            raw::fb_aux_text::MGA_STEP16 => Ok(TextType::MGAStep16),
            raw::fb_aux_text::MGA_STEP8 => Ok(TextType::MGAStep8),
            raw::fb_aux_text::SVGA_STEP2 => Ok(TextType::SVGAStep2),
            raw::fb_aux_text::SVGA_STEP4 => Ok(TextType::SVGAStep4),
            raw::fb_aux_text::SVGA_STEP8 => Ok(TextType::SVGAStep8),
            raw::fb_aux_text::SVGA_STEP16 => Ok(TextType::SVGAStep16),
            _ => Err(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VGAPlanesType {
    VGA4,
    CFB4,
    CFB8,
}

impl TryFrom<u32> for VGAPlanesType {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            raw::fb_aux_vga_planes::VGA4 => Ok(VGAPlanesType::VGA4),
            raw::fb_aux_vga_planes::CFB4 => Ok(VGAPlanesType::CFB4),
            raw::fb_aux_vga_planes::CFB8 => Ok(VGAPlanesType::CFB8),
            _ => Err(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    PackedPixels,
    Planes,
    InterleavedPlanes,
    Text(TextType),
    VGAPlanes(VGAPlanesType),
    FourCC,
}

impl TryFrom<(u32, u32)> for Type {
    type Error = (u32, u32);

    fn try_from(value: (u32, u32)) -> Result<Self, Self::Error> {
        match value {
            (raw::fb_type::PACKED_PIXELS, _) => Ok(Type::PackedPixels),
            (raw::fb_type::PLANES, _) => Ok(Type::Planes),
            (raw::fb_type::INTERLEAVED_PLANES, _) => Ok(Type::InterleavedPlanes),
            (raw::fb_type::TEXT, text_type) => {
                Ok(Type::Text(text_type.try_into().map_err(|_| value)?))
            }
            (raw::fb_type::VGA_PLANES, vga_planes_type) => Ok(Type::VGAPlanes(
                vga_planes_type.try_into().map_err(|_| value)?,
            )),
            (raw::fb_type::FOURCC, _) => Ok(Type::FourCC),
            _ => Err(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visual {
    Mono01,
    Mono10,
    TrueColor,
    PseudoColor,
    DirectColor,
    StaticPseudoColor,
    FourCC,
}

impl TryFrom<u32> for Visual {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            raw::fb_visual::MONO01 => Ok(Visual::Mono01),
            raw::fb_visual::MONO10 => Ok(Visual::Mono10),
            raw::fb_visual::TRUECOLOR => Ok(Visual::TrueColor),
            raw::fb_visual::PSEUDOCOLOR => Ok(Visual::PseudoColor),
            raw::fb_visual::DIRECTCOLOR => Ok(Visual::DirectColor),
            raw::fb_visual::STATIC_PSEUDOCOLOR => Ok(Visual::StaticPseudoColor),
            raw::fb_visual::FOURCC => Ok(Visual::FourCC),
            _ => Err(value),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FixedScreenInfo {
    pub id: [u8; 16],
    pub smem_start: c_ulong,
    pub smem_len: u32,
    type_: u32,
    type_aux: u32,
    visual: u32,
    xpanstep: u16,
    ypanstep: u16,
    ywrapstep: u16,
    pub line_length: u32,
    pub mmio_start: c_ulong,
    pub mmio_len: u32,
    pub accel: u32,
    pub capabilities: u16,
    _reserved: [u16; 2],
}

impl FixedScreenInfo {
    pub fn type_(&self) -> Result<Type, (u32, u32)> {
        (self.type_, self.type_aux).try_into()
    }

    pub fn visual(&self) -> Result<Visual, u32> {
        self.visual.try_into()
    }

    pub fn pan_step(&self) -> Option<(NonZeroU16, NonZeroU16)> {
        match (
            NonZeroU16::new(self.xpanstep),
            NonZeroU16::new(self.ypanstep),
        ) {
            (Some(x), Some(y)) => Some((x, y)),
            _ => None,
        }
    }

    pub fn ywrap_step(&self) -> Option<NonZeroU16> {
        NonZeroU16::new(self.ywrapstep)
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Bitfield {
    pub offset: u32,
    pub length: u32,
    msb_right: u32,
}

impl Bitfield {
    pub fn msb_right(&self) -> bool {
        self.msb_right != 0
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct VariableScreenInfo {
    pub xres: u32,
    pub yres: u32,
    pub xres_virtual: u32,
    pub yres_virtual: u32,
    pub xoffset: u32,
    pub yoffset: u32,
    pub bits_per_pixel: u32,
    grayscale: u32,
    pub red: Bitfield,
    pub green: Bitfield,
    pub blue: Bitfield,
    pub transp: Bitfield,
    nonstd: u32,
    activate: u32,
    pub height: u32,
    pub width: u32,
    pub accel_flags: u32,
    pub pixclock: u32,
    pub left_margin: u32,
    pub right_margin: u32,
    pub upper_margin: u32,
    pub lower_margin: u32,
    pub hsync_len: u32,
    pub vsync_len: u32,
    sync: u32,
    vmode: u32,
    rotate: u32,
    colorspace: u32,
    _reserved: [u32; 4],
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlankMode {
    /// Screen: unblanked, hsync: on, vsync: on
    Unblank = raw::vesa::NO_BLANKING,

    /// Screen: blanked, hsync: on, vsync: on
    Normal = raw::vesa::NO_BLANKING + 1,

    /// Screen: blanked, hsync: on, vsync: off
    VSyncSuspend = raw::vesa::VSYNC_SUSPEND + 1,

    /// Screen: blanked, hsync: off, vsync: on
    HSyncSuspend = raw::vesa::HSYNC_SUSPEND + 1,

    /// Screen: blanked, hsync: off, vsync: off
    Powerdown = raw::vesa::POWERDOWN + 1,
}

mod ioctl {
    use super::{FixedScreenInfo, VariableScreenInfo};

    const FBIOGET_VSCREENINFO: u16 = 0x4600;
    const FBIOPUT_VSCREENINFO: u16 = 0x4601;
    const FBIOGET_FSCREENINFO: u16 = 0x4602;
    const FBIOPAN_DISPLAY: u16 = 0x4606;
    const FBIOBLANK: u16 = 0x4611;

    nix::ioctl_read_bad!(fbioget_vscreeninfo, FBIOGET_VSCREENINFO, VariableScreenInfo);
    nix::ioctl_write_ptr_bad!(fbioput_vscreeninfo, FBIOPUT_VSCREENINFO, VariableScreenInfo);
    nix::ioctl_read_bad!(fbioget_fscreeninfo, FBIOGET_FSCREENINFO, FixedScreenInfo);
    nix::ioctl_write_ptr_bad!(fbiopan_display, FBIOPAN_DISPLAY, VariableScreenInfo);
    nix::ioctl_write_int_bad!(fbioblank, FBIOBLANK);
}

pub fn get_variable_screen_info(fd: c_int) -> Result<VariableScreenInfo, Errno> {
    let mut vscreeninfo = MaybeUninit::<VariableScreenInfo>::uninit();

    unsafe { ioctl::fbioget_vscreeninfo(fd, vscreeninfo.as_mut_ptr()) }?;

    Ok(unsafe { vscreeninfo.assume_init() })
}

pub fn set_variable_screen_info(fd: c_int, vscreeninfo: &VariableScreenInfo) -> Result<(), Errno> {
    unsafe { ioctl::fbioput_vscreeninfo(fd, vscreeninfo) }?;
    Ok(())
}

pub fn get_fixed_screen_info(fd: c_int) -> Result<FixedScreenInfo, Errno> {
    let mut fscreeninfo = MaybeUninit::<FixedScreenInfo>::uninit();

    unsafe { ioctl::fbioget_fscreeninfo(fd, fscreeninfo.as_mut_ptr()) }?;

    Ok(unsafe { fscreeninfo.assume_init() })
}

pub fn pan_display(fd: c_int, vscreeninfo: &VariableScreenInfo) -> Result<(), Errno> {
    unsafe { ioctl::fbiopan_display(fd, vscreeninfo) }?;
    Ok(())
}

pub fn set_blank_mode(fd: c_int, mode: BlankMode) -> Result<(), Errno> {
    unsafe { ioctl::fbioblank(fd, mode as i32) }?;
    Ok(())
}
