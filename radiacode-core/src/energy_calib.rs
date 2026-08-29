use radiacode_protocol::Error as ProtocolError;
use radiacode_protocol::{Command, Spectrum, VirtString, decode_spectrum};

use crate::device::RadiaCode;
use crate::error::{Error, Result};

pub async fn set_energy_calib(device: &mut RadiaCode, coefficients: [f32; 3]) -> Result<()> {
    let payload = [
        coefficients[0].to_le_bytes(),
        coefficients[1].to_le_bytes(),
        coefficients[2].to_le_bytes(),
    ]
    .concat();
    let mut args = Vec::with_capacity(12);
    args.extend_from_slice(&u32::from(VirtString::EnergyCalib).to_le_bytes());
    args.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    args.extend_from_slice(&payload);
    let mut response = device.execute_raw(Command::WrVirtString, &args).await?;
    let retcode = response.take_u32_le()?;
    if retcode != 1 {
        return Err(ProtocolError::UnexpectedReturnCode(retcode).into());
    }
    Ok(())
}

pub async fn spectrum_diff(device: &mut RadiaCode) -> Result<Spectrum> {
    let mut response = device.read_virt_string(VirtString::SpecDiff).await?;
    decode_spectrum(&mut response, device.spectrum_format_version()).map_err(Error::from)
}

impl RadiaCode {
    pub async fn set_energy_calib(&mut self, coefficients: [f32; 3]) -> Result<()> {
        set_energy_calib(self, coefficients).await
    }

    pub async fn spectrum_diff(&mut self) -> Result<Spectrum> {
        spectrum_diff(self).await
    }
}
