/// The reach, start chance and cooldown a decision reads for one motion or action.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CapabilityProfile {
    pub reach: Option<f32>,
    /// Probability that a winning finite motion actually starts.
    pub chance: f32,
    /// Seconds to wait after a finish or a missed start roll.
    pub cooldown: f32,
}

impl Default for CapabilityProfile {
    fn default() -> Self {
        Self {
            reach: None,
            chance: 1.0,
            cooldown: 0.0,
        }
    }
}
