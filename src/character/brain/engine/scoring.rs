//! How a consideration turns one raw input into a `0.0..=1.0` score.

use super::super::perception::ConsiderationInput;

/// An ordered `min < max` interval: a distance range, a consideration's input
/// range, or a span of seconds to roll from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    min: f32,
    max: f32,
}

impl Interval {
    /// Builds an interval, rejecting bad bounds at compile time.
    pub const fn new(min: f32, max: f32) -> Self {
        assert!(
            min.is_finite() && max.is_finite() && min < max,
            "interval needs finite min < max"
        );
        Self { min, max }
    }

    pub const fn min(self) -> f32 {
        self.min
    }

    pub const fn max(self) -> f32 {
        self.max
    }

    /// Signed distance outside the interval; zero inside it.
    pub fn error(self, value: f32) -> f32 {
        if value < self.min {
            value - self.min
        } else if value > self.max {
            value - self.max
        } else {
            0.0
        }
    }

    /// Offset from the center as a fraction of the half-width, clamped to `-1.0..=1.0`.
    pub fn center_error(self, value: f32) -> f32 {
        let half_width = (self.max - self.min) * 0.5;
        ((value - (self.min + self.max) * 0.5) / half_width).clamp(-1.0, 1.0)
    }

    /// Maps `raw` onto `0.0..=1.0` across the interval, clamping outside it.
    pub fn normalize_clamped(self, raw: f32) -> Option<f32> {
        if !raw.is_finite() {
            return None;
        }
        let normalized = ((raw - self.min) / (self.max - self.min)).clamp(0.0, 1.0);
        normalized.is_finite().then_some(normalized)
    }
}

/// One normalized input to a behavior's score.
#[derive(Clone, Copy, Debug)]
pub struct Consideration {
    pub name: &'static str,
    pub input: ConsiderationInput,
    pub score_mapping: ScoreMapping,
    pub inverted: bool,
}

impl Consideration {
    /// The opposite fact under a new name. A missing input still vetoes.
    pub const fn inverted(self, name: &'static str) -> Self {
        Self {
            name,
            inverted: !self.inverted,
            ..self
        }
    }
}

/// How a consideration turns its raw input into a `0.0..=1.0` score.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScoreMapping {
    /// Normalize over `range`, then shape with `curve`.
    Curve { range: Interval, curve: Curve },
    /// Full once the raw input crosses `enter`, zero before it. While its
    /// behavior holds a slot the threshold is `exit` instead, so the gap keeps
    /// the behavior. `exit < enter` passes raw inputs at or above the threshold;
    /// otherwise, including no gap at all, it passes raw inputs at or below it.
    Threshold { enter: f32, exit: f32 },
}

impl ScoreMapping {
    /// Score for `raw`, or `None` for a non-finite raw input.
    pub fn score_raw_input(self, raw: f32, is_current: bool) -> Option<f32> {
        match self {
            Self::Curve { range, curve } => {
                range.normalize_clamped(raw).map(|value| curve.apply(value))
            }
            Self::Threshold { enter, exit } => {
                if !raw.is_finite() {
                    return None;
                }
                let threshold = if is_current { exit } else { enter };
                let passes = if exit < enter {
                    raw >= threshold
                } else {
                    raw <= threshold
                };
                Some(if passes { 1.0 } else { 0.0 })
            }
        }
    }
}

/// Curve applied after normalizing a consideration's raw value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Curve {
    Linear,
    /// Steepens a partial input (`x^3`).
    Cubic,
}

impl Curve {
    pub(crate) fn apply(self, normalized: f32) -> f32 {
        let value = match self {
            Self::Linear => normalized,
            Self::Cubic => normalized * normalized * normalized,
        };

        value.clamp(0.0, 1.0)
    }
}
