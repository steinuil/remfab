use std::{
    ffi::c_ulong,
    fs::{self, File},
    io,
    mem::MaybeUninit,
    num::NonZeroU16,
    os::fd::AsRawFd as _,
    path::PathBuf,
};

use nix::errno::Errno;

use crate::rm2::sy7636a_temperature;

use super::fb_sys::*;

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
            fb_aux_text::MDA => Ok(TextType::MDA),
            fb_aux_text::CGA => Ok(TextType::CGA),
            fb_aux_text::S3_MMIO => Ok(TextType::S3MMIO),
            fb_aux_text::MGA_STEP16 => Ok(TextType::MGAStep16),
            fb_aux_text::MGA_STEP8 => Ok(TextType::MGAStep8),
            fb_aux_text::SVGA_STEP2 => Ok(TextType::SVGAStep2),
            fb_aux_text::SVGA_STEP4 => Ok(TextType::SVGAStep4),
            fb_aux_text::SVGA_STEP8 => Ok(TextType::SVGAStep8),
            fb_aux_text::SVGA_STEP16 => Ok(TextType::SVGAStep16),
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
            fb_aux_vga_planes::VGA4 => Ok(VGAPlanesType::VGA4),
            fb_aux_vga_planes::CFB4 => Ok(VGAPlanesType::CFB4),
            fb_aux_vga_planes::CFB8 => Ok(VGAPlanesType::CFB8),
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
            (fb_type::PACKED_PIXELS, _) => Ok(Type::PackedPixels),
            (fb_type::PLANES, _) => Ok(Type::Planes),
            (fb_type::INTERLEAVED_PLANES, _) => Ok(Type::InterleavedPlanes),
            (fb_type::TEXT, text_type) => Ok(Type::Text(text_type.try_into().map_err(|_| value)?)),
            (fb_type::VGA_PLANES, vga_planes_type) => Ok(Type::VGAPlanes(
                vga_planes_type.try_into().map_err(|_| value)?,
            )),
            (fb_type::FOURCC, _) => Ok(Type::FourCC),
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
            fb_visual::MONO01 => Ok(Visual::Mono01),
            fb_visual::MONO10 => Ok(Visual::Mono10),
            fb_visual::TRUECOLOR => Ok(Visual::TrueColor),
            fb_visual::PSEUDOCOLOR => Ok(Visual::PseudoColor),
            fb_visual::DIRECTCOLOR => Ok(Visual::DirectColor),
            fb_visual::STATIC_PSEUDOCOLOR => Ok(Visual::StaticPseudoColor),
            fb_visual::FOURCC => Ok(Visual::FourCC),
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
    Unblank = vesa::NO_BLANKING,

    /// Screen: blanked, hsync: on, vsync: on
    Normal = vesa::NO_BLANKING + 1,

    /// Screen: blanked, hsync: on, vsync: off
    VSyncSuspend = vesa::VSYNC_SUSPEND + 1,

    /// Screen: blanked, hsync: off, vsync: on
    HSyncSuspend = vesa::HSYNC_SUSPEND + 1,

    /// Screen: blanked, hsync: off, vsync: off
    Powerdown = vesa::POWERDOWN + 1,
}

mod raw_ioctl {
    use super::super::fb_sys::ioctl::*;
    use super::{FixedScreenInfo, VariableScreenInfo};

    nix::ioctl_read_bad!(fbioget_vscreeninfo, FBIOGET_VSCREENINFO, VariableScreenInfo);
    nix::ioctl_write_ptr_bad!(fbioput_vscreeninfo, FBIOPUT_VSCREENINFO, VariableScreenInfo);
    nix::ioctl_read_bad!(fbioget_fscreeninfo, FBIOGET_FSCREENINFO, FixedScreenInfo);
    nix::ioctl_write_ptr_bad!(fbiopan_display, FBIOPAN_DISPLAY, VariableScreenInfo);
    nix::ioctl_write_int_bad!(fbioblank, FBIOBLANK);
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ioctl 0x{0:x} failed: {1}")]
    Ioctl(u16, Errno),

    #[error("failed to read temperature: {0}")]
    Temperature(#[from] sy7636a_temperature::Error),
}

pub fn get_variable_screen_info(fd: &File) -> Result<VariableScreenInfo, Error> {
    let mut vscreeninfo = MaybeUninit::<VariableScreenInfo>::uninit();

    unsafe { raw_ioctl::fbioget_vscreeninfo(fd.as_raw_fd(), vscreeninfo.as_mut_ptr()) }
        .map_err(|errno| Error::Ioctl(ioctl::FBIOGET_VSCREENINFO, errno))?;

    Ok(unsafe { vscreeninfo.assume_init() })
}

pub fn set_variable_screen_info(fd: &File, vscreeninfo: &VariableScreenInfo) -> Result<(), Error> {
    unsafe { raw_ioctl::fbioput_vscreeninfo(fd.as_raw_fd(), vscreeninfo) }
        .map_err(|errno| Error::Ioctl(ioctl::FBIOPUT_VSCREENINFO, errno))?;
    Ok(())
}

pub fn get_fixed_screen_info(fd: &File) -> Result<FixedScreenInfo, Error> {
    let mut fscreeninfo = MaybeUninit::<FixedScreenInfo>::uninit();

    unsafe { raw_ioctl::fbioget_fscreeninfo(fd.as_raw_fd(), fscreeninfo.as_mut_ptr()) }
        .map_err(|errno| Error::Ioctl(ioctl::FBIOGET_FSCREENINFO, errno))?;

    Ok(unsafe { fscreeninfo.assume_init() })
}

pub fn pan_display(fd: &File, vscreeninfo: &VariableScreenInfo) -> Result<(), Error> {
    unsafe { raw_ioctl::fbiopan_display(fd.as_raw_fd(), vscreeninfo) }
        .map_err(|errno| Error::Ioctl(ioctl::FBIOPAN_DISPLAY, errno))?;
    Ok(())
}

pub fn set_blank_mode(fd: &File, mode: BlankMode) -> Result<(), Error> {
    unsafe { raw_ioctl::fbioblank(fd.as_raw_fd(), mode as i32) }
        .map_err(|errno| Error::Ioctl(ioctl::FBIOBLANK, errno))?;
    Ok(())
}

#[derive(Debug)]
pub struct Driver {
    fd: File,
    temperature_sensor: sy7636a_temperature::Sensor,
    front_buffer_index: i32,
    back_buffer_index: i32,
    var_screen_info: VariableScreenInfo,
}

impl Driver {
    pub fn start(&mut self) -> Result<(), Error> {
        set_blank_mode(&self.fd, BlankMode::Unblank)?;

        let temperature = self.temperature_sensor.read_temperature()?;

        let fscreeninfo = get_fixed_screen_info(&self.fd)?;
        let vscreeninfo = get_variable_screen_info(&self.fd)?;

        self.var_screen_info = vscreeninfo;

        Ok(())
    }

    pub fn page_flip(&mut self) -> Result<(), Error> {
        // self.fb_var_info.yoffset = self.back_buffer_index * self.dims.height;

        if self.front_buffer_index == -1 {
            set_variable_screen_info(&self.fd, &self.var_screen_info)
        } else {
            pan_display(&self.fd, &self.var_screen_info)
        }?;

        self.front_buffer_index = self.back_buffer_index;
        self.back_buffer_index = (self.back_buffer_index + 1) % 2;

        Ok(())
    }
}

const DEVICE_NAME: &[u8; 10] = b"mxs-lcdif\n";
const GRAPHICS_PATH: &str = "/sys/class/graphics";

pub fn discover_path() -> Result<Option<PathBuf>, io::Error> {
    for entry in fs::read_dir(GRAPHICS_PATH)? {
        let entry = entry?;

        let dir_path = entry.path();
        let name_path = dir_path.join("name");

        if !fs::exists(&name_path)? {
            continue;
        }

        if &fs::read(&name_path)? == DEVICE_NAME {
            let mut dev_path = PathBuf::new();
            dev_path.push("/dev");
            dev_path.push(entry.file_name());

            return Ok(Some(dev_path));
        }
    }

    Ok(None)
}
