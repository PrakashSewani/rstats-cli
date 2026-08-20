use super::ProcessSnapshot;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProcessSort {
    #[default]
    Cpu,
    Memory,
    Pid,
    Runtime,
    Name,
}

#[derive(Clone, Debug, Default)]
pub struct ProcessView {
    pub sort: ProcessSort,
    pub descending: bool,
    pub filter: String,
}

impl ProcessView {
    pub fn visible<'a>(&self, processes: &'a [ProcessSnapshot]) -> Vec<&'a ProcessSnapshot> {
        let filter = self.filter.to_lowercase();
        let mut visible: Vec<_> = processes
            .iter()
            .filter(|process| {
                filter.is_empty()
                    || process.name.to_lowercase().contains(&filter)
                    || process.command.to_lowercase().contains(&filter)
                    || process.pid.to_string() == filter
                    || process.user.to_lowercase().contains(&filter)
            })
            .collect();
        visible.sort_by(|left, right| {
            let ordering = match self.sort {
                ProcessSort::Cpu => left.cpu_usage.total_cmp(&right.cpu_usage),
                ProcessSort::Memory => left.memory_bytes.cmp(&right.memory_bytes),
                ProcessSort::Pid => left.pid.cmp(&right.pid),
                ProcessSort::Runtime => left.runtime_secs.cmp(&right.runtime_secs),
                ProcessSort::Name => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
            };
            let ordering = if self.descending { ordering.reverse() } else { ordering };
            ordering.then_with(|| left.pid.cmp(&right.pid))
        });
        visible
    }
}
