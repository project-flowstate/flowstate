//! Flowstate simulation kernel (stub).
//!
//! This is an intentionally minimal placeholder.

/// Simulation stub.
#[derive(Debug, Default)]
pub struct Sim;

impl Sim {
    /// Create a new simulation instance.
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::Sim;

    #[test]
    fn smoke_test() {
        let _sim = Sim::new();
    }
}
