use serde::Deserialize;
use std::num::NonZeroU64;
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
#[serde(from = "NonZeroU64")]
pub struct NonZeroDuration(Duration);

impl NonZeroDuration {
    pub fn new(duration: Duration) -> Option<Self> {
        if duration.is_zero() {
            None
        } else {
            Some(NonZeroDuration(duration))
        }
    }

    pub fn from_secs(secs: u64) -> Option<Self> {
        if secs == 0 {
            None
        } else {
            Some(Self(Duration::from_secs(secs)))
        }
    }

    pub fn from_non_zero_secs(secs: NonZeroU64) -> Self {
        Self(Duration::from_secs(secs.get()))
    }
}

impl From<NonZeroU64> for NonZeroDuration {
    fn from(secs: NonZeroU64) -> Self {
        Self::from_non_zero_secs(secs)
    }
}

impl From<NonZeroDuration> for Duration {
    fn from(duration: NonZeroDuration) -> Self {
        duration.0
    }
}
