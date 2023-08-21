use thread_local::*;

use std::cell::Cell;
use std::time::{Duration, Instant};

use crate::graph::*;

pub struct TimeNode {
    duration: ThreadLocal<Cell<Duration>>,
    graph: Graph,
}

impl TimeNode {
    const NAME: &'static str = "TimeNode";

    /// Track the runtime of the graph.
    pub fn new(graph: Graph) -> Self {
        Self {
            duration: ThreadLocal::new(),
            graph,
        }
    }

    /// Get the total time (in seconds) summed across all threads.
    pub fn total_time(&mut self) -> f64 {
        let duration = self.duration.iter_mut().map(|c| c.get()).sum::<Duration>();
        duration.as_secs_f64()
    }
}

impl GraphNode for TimeNode {
    fn run(&self, read: Option<Read>) -> Result<(Option<Read>, bool)> {
        let start = Instant::now();
        let res = self.graph.run_one(read);
        let elapsed = start.elapsed();
        let duration = self.duration.get_or(|| Cell::new(Duration::default()));
        duration.set(duration.get() + elapsed);
        res
    }

    fn required_names(&self) -> &[LabelOrAttr] {
        &[]
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }
}
