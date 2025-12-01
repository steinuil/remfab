use std::{
    io::{self, Read, Seek},
    ops::Index,
};

use crate::byte_reader::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    pub checksum: u32,
    pub filesize: u32,
    pub serial: u32,
    pub run_type: u8,
    pub fpl_platform: u8,
    pub fpl_lot: u16,
    pub adhesive_run: u8,
    pub waveform_version: u8,
    pub waveform_subversion: u8,
    pub waveform_type: u8,
    pub fpl_size: u8,
    pub mfg_code: u8,
    pub waveform_revision: u8,
    pub old_frame_rate: u8,
    pub frame_rate: u8,
    pub vcom_offset: u8,
    pub extra_info_addr: u32,
    pub checksum1: u8,
    pub wmta: u32,
    pub fvsn: u8,
    pub luts: u8,
    pub mode_count: u8,
    pub temp_range_count: u8,
    pub advanced_wfm_flags: u8,
    pub eb: u8,
    pub sb: u8,
    pub checksum2: u8,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid checksum for {field}: expected 0x{expected:x}, actual 0x{actual:x}")]
    InvalidChecksum {
        field: String,
        expected: u8,
        actual: u8,
    },

    #[error("invalid phase: 0b{0:08b}")]
    InvalidPhase(u8),

    #[error(transparent)]
    Read(#[from] io::Error),
}

fn header<R: Read + Seek>(input: &mut R) -> Result<Header, Error> {
    let checksum = le_u32(input)?;
    let filesize = le_u32(input)?;
    let serial = le_u32(input)?;
    let run_type = u8(input)?;
    let fpl_platform = u8(input)?;
    let fpl_lot = le_u16(input)?;
    let adhesive_run = u8(input)?;
    let waveform_version = u8(input)?;
    let waveform_subversion = u8(input)?;
    let waveform_type = u8(input)?;
    let fpl_size = u8(input)?;
    let mfg_code = u8(input)?;
    let waveform_revision = u8(input)?;
    let old_frame_rate = u8(input)?;
    let frame_rate = u8(input)?;
    let vcom_offset = u8(input)?;
    let _reserved = skip(2, input)?;
    let extra_info_addr = le_u24(input)?;
    let checksum1 = u8(input)?;
    let wmta = le_u24(input)?;
    let fvsn = u8(input)?;
    let luts = u8(input)?;
    let mode_count = u8(input)?;
    let temp_range_count = u8(input)?;
    let advanced_wfm_flags = u8(input)?;
    let eb = u8(input)?;
    let sb = u8(input)?;
    let _reserved = skip(5, input)?;
    let checksum2 = u8(input)?;

    let header = Header {
        checksum,
        filesize,
        serial,
        run_type,
        fpl_platform,
        fpl_lot,
        adhesive_run,
        waveform_version,
        waveform_subversion,
        waveform_type,
        fpl_size,
        mfg_code,
        waveform_revision,
        old_frame_rate,
        frame_rate,
        vcom_offset,
        extra_info_addr,
        checksum1,
        wmta,
        fvsn,
        luts,
        mode_count,
        temp_range_count,
        advanced_wfm_flags,
        eb,
        sb,
        checksum2,
    };

    Ok(header)
}

fn pointer<R: Read>(input: &mut R) -> Result<u32, Error> {
    let pointer = take_const(input)?;
    let checksum = u8(input)?;

    let actual: u8 = pointer[0]
        .overflowing_add(pointer[1])
        .0
        .overflowing_add(pointer[2])
        .0;
    if actual != checksum {
        return Err(Error::InvalidChecksum {
            field: "pointer".to_string(),
            expected: checksum,
            actual,
        });
    }

    let pointer = u24_from_le_bytes(pointer);
    Ok(pointer)
}

fn temperatures<R: Read>(count: usize, input: &mut R) -> Result<Vec<u8>, Error> {
    let temperatures = take(count + 2, input)?;
    let _checksum = u8(input)?;

    // TODO compute basic_checksum over the temperatures
    // if actual != checksum {
    //     return Err(nom::Err::Failure(Error::InvalidChecksum {
    //         field: "temperatures".to_string(),
    //         expected: checksum,
    //         actual,
    //     }));
    // }

    Ok(temperatures)
}

fn filename<R: Read>(input: &mut R) -> Result<Vec<u8>, Error> {
    let len = u8(input)?;
    let filename = take(len as usize, input)?;
    let _checksum = u8(input)?;

    // TODO compute basic_checksum over the filename
    // if actual != checksum {
    //     return Err(nom::Err::Failure(Error::InvalidChecksum {
    //         field: "filename".to_string(),
    //         expected: checksum,
    //         actual,
    //     }));
    // }

    Ok(filename)
}

fn find_waveform_blocks<R: Read + Seek>(
    mode_count: usize,
    temperatures_count: usize,
    input: &mut R,
) -> Result<Vec<u32>, Error> {
    let mut addresses = vec![];

    let start = input.seek(io::SeekFrom::Current(0))?;

    for i in 0..mode_count + 1 {
        let offset = pointer(input)?;

        input.seek(io::SeekFrom::Start(offset as u64))?;
        for _ in 0..temperatures_count + 1 {
            let address = pointer(input)?;

            addresses.push(address);
        }
        input.seek(io::SeekFrom::Start(start + (i as u64 * 4)))?;
    }

    Ok(addresses)
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Noop = 0b00,
    Black = 0b01,
    White = 0b10,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseCell(u8);

impl PhaseCell {
    pub fn new(phases: u8) -> Result<Self, Error> {
        if phases & 0b11000000 == 0b11000000
            || phases & 0b00110000 == 0b00110000
            || phases & 0b00001100 == 0b00001100
            || phases & 0b00000011 == 0b00000011
        {
            Err(Error::InvalidPhase(phases))
        } else {
            Ok(PhaseCell(phases))
        }
    }
}

impl Index<u8> for PhaseCell {
    type Output = Phase;

    fn index(&self, index: u8) -> &Self::Output {
        if index > 3 {
            panic!("index out of bounds for PhaseCell: {index}")
        }

        let shift = 6 - (2 * index);

        match (self.0 >> shift) & 0b11 {
            0b00 => &Phase::Noop,
            0b01 => &Phase::Black,
            0b10 => &Phase::White,
            phase => unreachable!("invalid phase: 0b{phase:02b}"),
        }
    }
}

const INTENSITY_VALUES: usize = 1 << 5;

type PhaseMatrix = Box<[[Phase; INTENSITY_VALUES]; INTENSITY_VALUES]>;

fn waveform<R: Read + Seek>(end: u64, input: &mut R) -> Result<Vec<PhaseMatrix>, Error> {
    let mut block = vec![];

    let mut matrix = Box::new([[Phase::Noop; INTENSITY_VALUES]; INTENSITY_VALUES]);

    let mut i = 0;
    let mut j = 0;
    let mut repeat_mode = true;

    loop {
        let pos = input.seek(io::SeekFrom::Current(0))?;
        if pos == end {
            break;
        }

        let phases = u8(input)?;

        if phases == 0xFC {
            repeat_mode = !repeat_mode;
            continue;
        }

        if phases == 0xFF {
            break;
        }

        let mut repeat = 1;

        if repeat_mode {
            repeat = u8(input)? as u32 + 1;
        }

        let phase_cell = PhaseCell::new(phases)?;

        for _ in 0..repeat {
            matrix[j][i] = phase_cell[0];
            matrix[j + 1][i] = phase_cell[1];
            matrix[j + 2][i] = phase_cell[2];
            matrix[j + 3][i] = phase_cell[3];

            j += 4;

            if j == INTENSITY_VALUES {
                j = 0;
                i += 1;
            }

            if i == INTENSITY_VALUES {
                i = 0;
                block.push(matrix);
                matrix = Box::new([[Phase::Noop; INTENSITY_VALUES]; INTENSITY_VALUES])
            }
        }
    }

    Ok(block)
}

#[test]
fn parse_pointer_test() {
    use std::io::Cursor;

    let mut input = Cursor::new(vec![0x5, 0x5, 0x6, 0x10]);

    let p = pointer(&mut input).unwrap();

    assert_eq!(p, 0x060505);
}

#[test]
fn parse_test() {
    use std::fs::File;
    use std::io::SeekFrom;

    let mut input = File::open("320_R467_AF4731_ED103TC2C6_VB3300-KCD_TC.wbf").unwrap();

    let header = header(&mut input).unwrap();
    let temperatures = temperatures(header.temp_range_count as usize, &mut input).unwrap();
    let filename = filename(&mut input).unwrap();
    let blocks = find_waveform_blocks(
        header.mode_count as usize,
        header.temp_range_count as usize,
        &mut input,
    )
    .unwrap();

    input.seek(SeekFrom::Start(blocks[0] as u64)).unwrap();
    let end = blocks[1] as u64;
    let waveform = waveform(end, &mut input).unwrap();

    println!("{:?}", waveform.last().unwrap());
    println!("{:#?}", waveform.len());

    assert!(false);
}
