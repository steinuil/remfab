use nix::{
    errno::Errno,
    libc::{self, c_int},
};

#[repr(u32)]
enum FbType {
    PackedPixels = 0,
    Planes = 1,
    InterleavedPlanes = 2,
    Text = 3,
    VGAPlanes = 4,
    FourCC = 5,
}

#[repr(u32)]
enum FbVisual {
    Mono01 = 0,
    Mono10 = 1,
    TrueColor = 2,
    PseudoColor = 3,
    DirectColor = 4,
    StaticPseudoColor = 5,
    FourCC = 6,
}

#[repr(C)]
struct FbFixScreeninfo {
    pub id: [u8; 16],
    pub smem_start: libc::c_ulong,
    pub smem_len: u32,
    pub type_: FbType,
    pub type_aux: u32,
    pub visual: FbVisual,
    pub xpanstep: u16,
    pub ypanstep: u16,
    pub ywrapstep: u16,
    pub line_length: u32,
    pub mmio_start: libc::c_ulong,
    pub mmio_len: u32,
    pub accel: u32,
    pub capabilities: u16,
    pub _reserved: [u16; 2],
}

const FBIOGET_VSCREENINFO: u16 = 0x4600;
const FBIOPUT_VSCREENINFO: u16 = 0x4601;
const FBIOGET_FSCREENINFO: u16 = 0x4602;
const FBIOBLANK: u16 = 0x4611;

nix::ioctl_read_bad!(fbioget_fscreeninfo, FBIOGET_FSCREENINFO, FbFixScreeninfo);
nix::ioctl_write_int_bad!(fbio_blank, FBIOBLANK);

mod vesa_blank_mode {
    pub const NO_BLANKING: i32 = 0;
    pub const VSYNC_SUSPEND: i32 = 1;
    pub const HSYNC_SUSPEND: i32 = 2;
    pub const POWERDOWN: i32 = VSYNC_SUSPEND | HSYNC_SUSPEND;
}

#[repr(i32)]
enum BlankMode {
    Unblank = vesa_blank_mode::NO_BLANKING,
    Normal = vesa_blank_mode::NO_BLANKING + 1,
    VSyncSuspend = vesa_blank_mode::VSYNC_SUSPEND + 1,
    HSyncSuspend = vesa_blank_mode::HSYNC_SUSPEND + 1,
    Powerdown = vesa_blank_mode::POWERDOWN + 1,
}

fn blank(fd: c_int, mode: BlankMode) -> Result<(), Errno> {
    unsafe { fbio_blank(fd, mode as i32)? };
    Ok(())
}

fn main() {}
