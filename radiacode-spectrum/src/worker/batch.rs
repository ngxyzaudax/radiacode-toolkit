use tokio::sync::mpsc::UnboundedReceiver;

use super::messages::WorkerCommand;

#[derive(Debug, Default)]
pub struct CoalescedBatch {
    pub priority: Option<WorkerCommand>,
    pub fetch_monitor: bool,
    pub fetch_spectrum: bool,
}

impl CoalescedBatch {
    pub fn monitor_only() -> Self {
        Self {
            fetch_monitor: true,
            ..Self::default()
        }
    }

    pub fn monitor_and_spectrum() -> Self {
        Self {
            fetch_monitor: true,
            fetch_spectrum: true,
            ..Self::default()
        }
    }

    pub fn absorb(&mut self, command: WorkerCommand) {
        match command {
            WorkerCommand::FetchMonitor => self.fetch_monitor = true,
            WorkerCommand::FetchSpectrum => self.fetch_spectrum = true,
            priority => self.priority = Some(priority),
        }
    }

    pub fn has_fetch(&self) -> bool {
        self.fetch_monitor || self.fetch_spectrum
    }
}

pub async fn recv_batch(commands: &mut UnboundedReceiver<WorkerCommand>) -> Option<CoalescedBatch> {
    let first = commands.recv().await?;
    let mut batch = CoalescedBatch::default();
    batch.absorb(first);
    while batch.priority.is_none() {
        match commands.try_recv() {
            Ok(next) => batch.absorb(next),
            Err(_) => break,
        }
    }
    Some(batch)
}

#[cfg(test)]
mod tests {
    use super::{CoalescedBatch, recv_batch};
    use crate::worker::WorkerCommand;
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test]
    async fn coalesces_fetch_commands_until_priority_command() {
        let (tx, mut rx) = unbounded_channel();
        tx.send(WorkerCommand::FetchMonitor).unwrap();
        tx.send(WorkerCommand::FetchSpectrum).unwrap();
        tx.send(WorkerCommand::FetchMonitor).unwrap();
        tx.send(WorkerCommand::Scan).unwrap();
        tx.send(WorkerCommand::FetchSpectrum).unwrap();
        let batch = recv_batch(&mut rx).await.expect("batch");
        assert!(batch.fetch_monitor);
        assert!(batch.fetch_spectrum);
        assert!(matches!(batch.priority, Some(WorkerCommand::Scan)));
        let remaining = recv_batch(&mut rx).await.expect("second batch");
        assert!(remaining.fetch_spectrum);
        assert!(!remaining.fetch_monitor);
        assert!(remaining.priority.is_none());
    }

    #[tokio::test]
    async fn closed_channel_ends_batching() {
        let (tx, mut rx) = unbounded_channel::<WorkerCommand>();
        drop(tx);
        assert!(recv_batch(&mut rx).await.is_none());
    }

    #[test]
    fn preset_batches_report_fetch_flags() {
        assert!(CoalescedBatch::monitor_only().has_fetch());
        assert!(!CoalescedBatch::monitor_only().fetch_spectrum);
        assert!(CoalescedBatch::monitor_and_spectrum().fetch_spectrum);
        assert!(!CoalescedBatch::default().has_fetch());
    }
}
