use std::collections::VecDeque;
use std::time::Duration;

use super::state::MonitorSample;

const HISTORY_MINUTES: f64 = 60.0;
const MAX_SAMPLES: usize = 3900;

pub fn trim_history(history: &mut VecDeque<MonitorSample>, elapsed: Duration) {
    let window = Duration::from_secs_f64(HISTORY_MINUTES * 60.0);
    while history.len() > MAX_SAMPLES {
        history.pop_front();
    }
    while history
        .front()
        .is_some_and(|sample| elapsed.saturating_sub(sample.elapsed) > window)
    {
        history.pop_front();
    }
}
