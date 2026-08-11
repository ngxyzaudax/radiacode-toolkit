use std::time::Duration;

use tokio::time::sleep;
use tracing::debug;

use radiacode_protocol::{
    decode_fw_version, decode_spectrum, Command, Spectrum, VirtString,
};
use radiacode_protocol::Error as ProtocolError;

use crate::device::RadiaCode;
use crate::error::{Error, Result};
use radiacode_protocol::DeviceVersions;

const FW_VERSION_ATTEMPTS: usize = 3;

pub async fn fw_version(device: &mut RadiaCode) -> Result<DeviceVersions> {
    let mut last_error: Option<Error> = None;
    for attempt in 0..FW_VERSION_ATTEMPTS {
        if attempt > 0 {
            debug!(attempt, "retrying fw_version after transient parse error");
            sleep(Duration::from_millis(250)).await;
        }
        device.drain_transport().await;
        let response = device.execute_raw(Command::GetVersion, &[]).await?;
        match decode_fw_version(response) {
            Ok(versions) => return Ok(versions),
            Err(error) if error.is_transient() => last_error = Some(error.into()),
            Err(error) => return Err(error.into()),
        }
    }
    Err(last_error.unwrap_or(ProtocolError::Timeout.into()))
}

pub async fn hw_serial_number(device: &mut RadiaCode) -> Result<String> {
    let mut response = device.execute_raw(Command::GetSerial, &[]).await?;
    let serial_len = response.take_u32_le()? as usize;
    let mut groups = Vec::new();
    for _ in 0..(serial_len / 4) {
        groups.push(format!("{:08X}", response.take_u32_le()?));
    }
    Ok(groups.join("-"))
}

pub async fn serial_number(device: &mut RadiaCode) -> Result<String> {
    let response = device.read_virt_string(VirtString::SerialNumber).await?;
    Ok(String::from_utf8_lossy(response.data()).into_owned())
}

pub async fn configuration(device: &mut RadiaCode) -> Result<String> {
    let response = device.read_virt_string(VirtString::Configuration).await?;
    Ok(String::from_utf8_lossy(response.data()).into_owned())
}

pub async fn spectrum(device: &mut RadiaCode) -> Result<Spectrum> {
    let mut response = device.read_virt_string(VirtString::Spectrum).await?;
    decode_spectrum(&mut response, device.spectrum_format_version).map_err(Error::from)
}

pub async fn spectrum_accum(device: &mut RadiaCode) -> Result<Spectrum> {
    let mut response = device.read_virt_string(VirtString::SpecAccum).await?;
    decode_spectrum(&mut response, device.spectrum_format_version).map_err(Error::from)
}

pub async fn energy_calib(device: &mut RadiaCode) -> Result<[f32; 3]> {
    let mut response = device.read_virt_string(VirtString::EnergyCalib).await?;
    Ok([
        response.take_f32_le()?,
        response.take_f32_le()?,
        response.take_f32_le()?,
    ])
}

impl RadiaCode {
    pub async fn fw_version(&mut self) -> Result<DeviceVersions> {
        fw_version(self).await
    }

    pub async fn hw_serial_number(&mut self) -> Result<String> {
        hw_serial_number(self).await
    }

    pub async fn serial_number(&mut self) -> Result<String> {
        serial_number(self).await
    }

    pub async fn configuration(&mut self) -> Result<String> {
        configuration(self).await
    }

    pub async fn spectrum(&mut self) -> Result<Spectrum> {
        spectrum(self).await
    }

    pub async fn spectrum_accum(&mut self) -> Result<Spectrum> {
        spectrum_accum(self).await
    }

    pub async fn energy_calib(&mut self) -> Result<[f32; 3]> {
        energy_calib(self).await
    }
}
