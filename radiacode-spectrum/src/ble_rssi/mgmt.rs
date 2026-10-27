use std::io;
use std::time::Duration;

use bdaddr::Address;
use btmgmt::Client;
use btmgmt::command::GetConnectionInformation;
use btmgmt::packet::ErrorCode;
use futures::StreamExt;
use tokio::time::timeout;
use tracing::debug;

use super::MgmtRssi;

const MGMT_RSSI_TIMEOUT: Duration = Duration::from_millis(400);
const MGMT_RSSI_UNKNOWN: i8 = 127;

pub async fn read_connected_rssi(mac: &str) -> MgmtRssi {
    timeout(MGMT_RSSI_TIMEOUT, probe_addresses(mac))
        .await
        .unwrap_or(MgmtRssi::Unavailable)
}

async fn probe_addresses(mac: &str) -> MgmtRssi {
    let public = read_for_address(mac, Address::le_public_from_str(mac).ok()).await;
    match public {
        MgmtRssi::Unavailable => read_for_address(mac, Address::le_random_from_str(mac).ok()).await,
        status => status,
    }
}

async fn read_for_address(mac: &str, address: Option<Address>) -> MgmtRssi {
    let Some(address) = address else {
        return MgmtRssi::Unavailable;
    };
    let client = match Client::open() {
        Ok(client) => client,
        Err(error) if is_permission_denied(&error) => return MgmtRssi::PermissionDenied,
        Err(_) => return MgmtRssi::Unavailable,
    };
    let mut events = client.events().await;
    tokio::spawn(async move { while events.next().await.is_some() {} });
    match client
        .call(None, GetConnectionInformation::new(address))
        .await
    {
        Ok(reply) => measured_rssi(mac, *reply.rssi() as i8),
        Err(error) if is_permission_denied(&error) => MgmtRssi::PermissionDenied,
        Err(_) => MgmtRssi::Unavailable,
    }
}

fn measured_rssi(mac: &str, rssi: i8) -> MgmtRssi {
    if rssi == MGMT_RSSI_UNKNOWN {
        return MgmtRssi::Unavailable;
    }
    debug!(%mac, rssi, "rssi from mgmt conn-info");
    MgmtRssi::Measured(i16::from(rssi))
}

pub(super) fn is_permission_denied(error: &btmgmt::client::Error) -> bool {
    match error {
        btmgmt::client::Error::Reply(ErrorCode::PermissionDenied) => true,
        btmgmt::client::Error::Io(error) => error.kind() == io::ErrorKind::PermissionDenied,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::is_permission_denied;
    use btmgmt::packet::ErrorCode;

    #[test]
    fn classifies_management_reply_and_socket_denial() {
        let reply = btmgmt::client::Error::Reply(ErrorCode::PermissionDenied);
        let socket = btmgmt::client::Error::Io(std::io::Error::from_raw_os_error(13));
        let rejected = btmgmt::client::Error::Reply(ErrorCode::Rejected);
        assert!(is_permission_denied(&reply));
        assert!(is_permission_denied(&socket));
        assert!(!is_permission_denied(&rejected));
    }
}
