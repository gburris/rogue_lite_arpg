use rand::RngExt;

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

    /// Both bounds multiplied by `factor`, which must be positive to keep `min < max`.
    pub fn scaled(self, factor: f32) -> Self {
        assert!(
            factor > 0.0,
            "scaling an interval by {factor} would leave it empty or reversed"
        );
        Self::new(self.min * factor, self.max * factor)
    }

    /// A uniformly random value from `min` up to, not including, `max`.
    pub fn pick_random_value(self) -> f32 {
        rand::rng().random_range(self.min..self.max)
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
        ((value - self.min.midpoint(self.max)) / half_width).clamp(-1.0, 1.0)
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

/// `value` as a fraction of `max`, clamped to `0.0..=1.0`; `None` unless both are finite and `max`
/// is positive.
pub fn fraction_of_max(value: f32, max: f32) -> Option<f32> {
    (value.is_finite() && max.is_finite() && max > 0.0).then(|| (value / max).clamp(0.0, 1.0))
}

/// A range centered on zero, `-half_width..=half_width`: an angle to scatter a direction by, or
/// seconds to jitter a wait by. Unlike an [`Interval`], it may have zero width.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Spread {
    half_width: f32,
}

impl Spread {
    /// Builds a spread; panics on a negative or infinite half-width, at compile time in a const.
    pub const fn new(half_width: f32) -> Self {
        assert!(
            half_width.is_finite() && half_width >= 0.0,
            "a spread samples from -half_width..=half_width, which is reversed or unbounded"
        );
        Self { half_width }
    }

    /// A uniformly random value from `-half_width` to `half_width`; zero for a zero-width spread.
    pub fn pick_random_value(self) -> f32 {
        if self.half_width == 0.0 {
            return 0.0;
        }
        rand::rng().random_range(-self.half_width..=self.half_width)
    }
}
