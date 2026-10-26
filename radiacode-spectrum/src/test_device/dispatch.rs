use radiacode_lib::Result;
use radiacode_lib::protocol::{BytesBuffer, Command};

use super::encode::{
    fw_signature_payload, fw_version_payload, hw_serial_payload, u32_words, virt_string_payload,
    vsfr_read_payload,
};
use super::virtual_device::VirtualDevice;

pub fn respond(device: &mut VirtualDevice, opcode: u16, args: &[u8]) -> Result<Vec<u8>> {
    let mut args = BytesBuffer::new(args.to_vec());
    match Command::from_opcode(opcode) {
        Some(Command::GetStatus) => Ok(device.status_word.to_le_bytes().to_vec()),
        Some(Command::GetVersion) => Ok(fw_version_payload(
            device.boot_firmware,
            device.target_firmware,
        )),
        Some(Command::GetSerial) => Ok(hw_serial_payload(&device.hw_serial_groups)),
        Some(Command::FwSignature) => Ok(fw_signature_payload(0xCAFE, "rc-103.fw", "RadiaCode")),
        Some(Command::RdVirtString) => read_virt_string(device, &mut args),
        Some(Command::WrVirtString) => write_virt_string(device, &mut args),
        Some(Command::RdVirtSfr) => read_vsfr(device, &mut args),
        Some(Command::WrVirtSfr) => write_vsfr(device, &mut args),
        Some(Command::RdVirtSfrBatch) => read_vsfr_batch(device, &mut args),
        Some(Command::WrVirtSfrBatch) => write_vsfr_batch(device, &mut args),
        _ => Ok(Vec::new()),
    }
}

fn read_virt_string(device: &VirtualDevice, args: &mut BytesBuffer) -> Result<Vec<u8>> {
    let id = args.take_u32_le()?;
    Ok(device.strings.get(&id).map_or_else(
        || virt_string_payload(0, &[]),
        |data| virt_string_payload(1, data),
    ))
}

fn write_virt_string(device: &mut VirtualDevice, args: &mut BytesBuffer) -> Result<Vec<u8>> {
    let id = args.take_u32_le()?;
    let length = args.take_len_u32()?;
    let data = args.take_bytes(length)?.to_vec();
    device.strings.insert(id, data);
    Ok(u32_words(&[1]))
}

fn read_vsfr(device: &VirtualDevice, args: &mut BytesBuffer) -> Result<Vec<u8>> {
    let id = args.take_u32_le()?;
    Ok(vsfr_read_payload(device.registers.get(&id).copied()))
}

fn write_vsfr(device: &mut VirtualDevice, args: &mut BytesBuffer) -> Result<Vec<u8>> {
    let id = args.take_u32_le()?;
    let value = if args.size() == 4 {
        args.take_u32_le()?
    } else {
        0
    };
    device.registers.insert(id, value);
    Ok(u32_words(&[1]))
}

fn read_vsfr_batch(device: &VirtualDevice, args: &mut BytesBuffer) -> Result<Vec<u8>> {
    if !device.batch_supported {
        return Ok(u32_words(&[0]));
    }
    let ids = take_ids(args)?;
    let present: Vec<(usize, u32)> = ids
        .iter()
        .enumerate()
        .filter_map(|(index, id)| device.registers.get(id).map(|value| (index, *value)))
        .collect();
    let valid_flags = present
        .iter()
        .fold(0u32, |flags, (index, _)| flags | (1 << index));
    let values: Vec<u32> = present.iter().map(|(_, value)| *value).collect();
    Ok([u32_words(&[valid_flags]), u32_words(&values)].concat())
}

fn write_vsfr_batch(device: &mut VirtualDevice, args: &mut BytesBuffer) -> Result<Vec<u8>> {
    let ids = take_ids(args)?;
    for id in &ids {
        let value = args.take_u32_le()?;
        device.registers.insert(*id, value);
    }
    Ok(u32_words(&[(1u32 << ids.len()) - 1]))
}

fn take_ids(args: &mut BytesBuffer) -> Result<Vec<u32>> {
    let count = args.take_len_u32()?;
    (0..count).map(|_| args.take_u32_le()).collect()
}
