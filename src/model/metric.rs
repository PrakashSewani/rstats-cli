use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MetricKind {
    Cpu,
    Memory,
    Swap,
    LoadAverage,
    NetworkReceive,
    NetworkTransmit,
    Disk,
}

impl fmt::Display for MetricKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Cpu => "cpu.total",
            Self::Memory => "memory.used_percent",
            Self::Swap => "swap.used_percent",
            Self::LoadAverage => "load.average",
            Self::NetworkReceive => "network.receive_bytes",
            Self::NetworkTransmit => "network.transmit_bytes",
            Self::Disk => "disk.max_used_percent",
        };
        formatter.write_str(value)
    }
}

#[derive(Clone, Debug, Default)]
pub struct MetricHistory {
    values: Vec<f64>,
    capacity: usize,
}

impl MetricHistory {
    pub fn new(capacity: usize) -> Self {
        Self { values: Vec::with_capacity(capacity), capacity }
    }

    pub fn push(&mut self, value: f64) {
        if self.capacity == 0 {
            return;
        }
        if self.values.len() == self.capacity {
            self.values.remove(0);
        }
        self.values.push(value);
    }

    pub fn values(&self) -> &[f64] {
        &self.values
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
