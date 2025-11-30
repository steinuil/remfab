use std::{
    fs::{self, File},
    io::{self, Read as _, Seek as _},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const READING_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Read(#[from] io::Error),

    #[error("temperature sensor returned a garbage value: {0:?}")]
    InvalidTemperature(String),

    #[error("display is powered off")]
    PoweredOff,
}

#[derive(Debug)]
struct Reading {
    pub temperature_celsius: u8,
    pub timestamp: Instant,
}

#[derive(Debug)]
/// A wrapper for reading the temperature reported by the SY7636A PMIC.
pub struct Sensor {
    fd: File,
    last_reading: Option<Reading>,
}

impl Sensor {
    pub fn open_path<P: AsRef<Path>>(path: &P) -> Result<Self, io::Error> {
        let fd = File::open(path)?;

        Ok(Sensor {
            fd,
            last_reading: None,
        })
    }

    /// Read the temperature in Celsius.
    pub fn read_temperature(&mut self) -> Result<u8, Error> {
        let now = Instant::now();

        match &self.last_reading {
            Some(reading) if now - reading.timestamp < READING_INTERVAL => {
                return Ok(reading.temperature_celsius);
            }
            None | Some(_) => {}
        }

        self.fd.seek(io::SeekFrom::Start(0))?;

        let mut temp_str = String::new();
        self.fd.read_to_string(&mut temp_str)?;

        let temperature_celsius = temp_str
            .parse()
            .map_err(|_| Error::InvalidTemperature(temp_str))?;

        if temperature_celsius > 0 {
            self.last_reading = Some(Reading {
                temperature_celsius,
                timestamp: now,
            });

            Ok(temperature_celsius)
        } else {
            Err(Error::PoweredOff)
        }
    }
}

const DEVICE_NAME: &[u8; 19] = b"sy7636a_temperature";
const HWMON_PATH: &str = "/sys/class/hwmon";

pub fn discover_path() -> Result<Option<PathBuf>, io::Error> {
    for entry in fs::read_dir(HWMON_PATH)? {
        let entry = entry?;
        let dir_path = entry.path();
        let name_path = dir_path.join("name");

        if !fs::exists(&name_path)? {
            continue;
        }

        if &fs::read(name_path)? != DEVICE_NAME {
            continue;
        }

        let temp_path = dir_path.join("temp0");

        if fs::exists(&temp_path)? {
            return Ok(Some(temp_path));
        }
    }

    Ok(None)
}
