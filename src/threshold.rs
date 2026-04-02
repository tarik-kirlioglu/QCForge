use std::path::Path;

use serde::Deserialize;

use crate::error::{QcForgeError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ThresholdDirection {
    HigherIsBetter,
    LowerIsBetter,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetricThreshold {
    pub warn: f64,
    pub fail: f64,
    pub direction: ThresholdDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QcLevel {
    Pass,
    Warn,
    Fail,
}

impl QcLevel {
    pub fn worst(self, other: QcLevel) -> QcLevel {
        match (self, other) {
            (QcLevel::Fail, _) | (_, QcLevel::Fail) => QcLevel::Fail,
            (QcLevel::Warn, _) | (_, QcLevel::Warn) => QcLevel::Warn,
            _ => QcLevel::Pass,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            QcLevel::Pass => "PASS",
            QcLevel::Warn => "WARN",
            QcLevel::Fail => "FAIL",
        }
    }
}

impl MetricThreshold {
    pub fn evaluate(&self, value: f64) -> QcLevel {
        match self.direction {
            ThresholdDirection::HigherIsBetter => {
                if value < self.fail {
                    QcLevel::Fail
                } else if value < self.warn {
                    QcLevel::Warn
                } else {
                    QcLevel::Pass
                }
            }
            ThresholdDirection::LowerIsBetter => {
                if value > self.fail {
                    QcLevel::Fail
                } else if value > self.warn {
                    QcLevel::Warn
                } else {
                    QcLevel::Pass
                }
            }
        }
    }

    pub fn color(&self, value: f64) -> ratatui::style::Color {
        match self.evaluate(value) {
            QcLevel::Pass => ratatui::style::Color::Green,
            QcLevel::Warn => ratatui::style::Color::Yellow,
            QcLevel::Fail => ratatui::style::Color::Red,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThresholdConfig {
    pub mapping_rate: MetricThreshold,
    pub duplication_rate: MetricThreshold,
    pub error_rate: MetricThreshold,
    pub ts_tv_ratio: MetricThreshold,
    pub gc_deviation: MetricThreshold,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            mapping_rate: MetricThreshold {
                warn: 90.0,
                fail: 80.0,
                direction: ThresholdDirection::HigherIsBetter,
            },
            duplication_rate: MetricThreshold {
                warn: 15.0,
                fail: 30.0,
                direction: ThresholdDirection::LowerIsBetter,
            },
            error_rate: MetricThreshold {
                warn: 0.005,
                fail: 0.01,
                direction: ThresholdDirection::LowerIsBetter,
            },
            ts_tv_ratio: MetricThreshold {
                warn: 1.8,
                fail: 1.5,
                direction: ThresholdDirection::HigherIsBetter,
            },
            gc_deviation: MetricThreshold {
                warn: 15.0,
                fail: 25.0,
                direction: ThresholdDirection::LowerIsBetter,
            },
        }
    }
}

impl ThresholdConfig {
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| QcForgeError::TomlParse(e.to_string()))
    }

    /// Evaluate a sample across all applicable metrics.
    /// Returns the worst QcLevel found. None values are skipped.
    pub fn evaluate_sample(
        &self,
        mapping_rate: Option<f64>,
        duplication_rate: Option<f64>,
        error_rate: Option<f64>,
        ts_tv_ratio: Option<f64>,
        gc_percent: Option<f64>,
    ) -> QcLevel {
        let mut level = QcLevel::Pass;

        if let Some(v) = mapping_rate {
            level = level.worst(self.mapping_rate.evaluate(v));
        }
        if let Some(v) = duplication_rate {
            level = level.worst(self.duplication_rate.evaluate(v));
        }
        if let Some(v) = error_rate {
            level = level.worst(self.error_rate.evaluate(v));
        }
        if let Some(v) = ts_tv_ratio {
            level = level.worst(self.ts_tv_ratio.evaluate(v));
        }
        if let Some(v) = gc_percent {
            let deviation = (v - 50.0).abs();
            level = level.worst(self.gc_deviation.evaluate(deviation));
        }

        level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_higher_is_better_pass() {
        let t = MetricThreshold {
            warn: 90.0,
            fail: 80.0,
            direction: ThresholdDirection::HigherIsBetter,
        };
        assert_eq!(t.evaluate(95.0), QcLevel::Pass);
    }

    #[test]
    fn test_higher_is_better_warn() {
        let t = MetricThreshold {
            warn: 90.0,
            fail: 80.0,
            direction: ThresholdDirection::HigherIsBetter,
        };
        assert_eq!(t.evaluate(85.0), QcLevel::Warn);
    }

    #[test]
    fn test_higher_is_better_fail() {
        let t = MetricThreshold {
            warn: 90.0,
            fail: 80.0,
            direction: ThresholdDirection::HigherIsBetter,
        };
        assert_eq!(t.evaluate(70.0), QcLevel::Fail);
    }

    #[test]
    fn test_lower_is_better_pass() {
        let t = MetricThreshold {
            warn: 15.0,
            fail: 30.0,
            direction: ThresholdDirection::LowerIsBetter,
        };
        assert_eq!(t.evaluate(5.0), QcLevel::Pass);
    }

    #[test]
    fn test_lower_is_better_fail() {
        let t = MetricThreshold {
            warn: 15.0,
            fail: 30.0,
            direction: ThresholdDirection::LowerIsBetter,
        };
        assert_eq!(t.evaluate(40.0), QcLevel::Fail);
    }

    #[test]
    fn test_default_thresholds() {
        let config = ThresholdConfig::default();
        assert_eq!(config.mapping_rate.evaluate(95.0), QcLevel::Pass);
        assert_eq!(config.mapping_rate.evaluate(85.0), QcLevel::Warn);
        assert_eq!(config.mapping_rate.evaluate(70.0), QcLevel::Fail);
        assert_eq!(config.duplication_rate.evaluate(5.0), QcLevel::Pass);
        assert_eq!(config.duplication_rate.evaluate(20.0), QcLevel::Warn);
        assert_eq!(config.duplication_rate.evaluate(40.0), QcLevel::Fail);
    }

    #[test]
    fn test_evaluate_sample_worst_wins() {
        let config = ThresholdConfig::default();
        // Good mapping but bad duplication → FAIL
        let level = config.evaluate_sample(
            Some(95.0),  // PASS
            Some(40.0),  // FAIL
            Some(0.001), // PASS
            None,
            None,
        );
        assert_eq!(level, QcLevel::Fail);
    }

    #[test]
    fn test_evaluate_sample_all_none() {
        let config = ThresholdConfig::default();
        let level = config.evaluate_sample(None, None, None, None, None);
        assert_eq!(level, QcLevel::Pass);
    }

    #[test]
    fn test_gc_deviation() {
        let config = ThresholdConfig::default();
        // GC 30% → deviation 20 → WARN (>15)
        let level = config.evaluate_sample(None, None, None, None, Some(30.0));
        assert_eq!(level, QcLevel::Warn);
        // GC 20% → deviation 30 → FAIL (>25)
        let level = config.evaluate_sample(None, None, None, None, Some(20.0));
        assert_eq!(level, QcLevel::Fail);
    }

    #[test]
    fn test_load_toml() {
        let toml_str = r#"
[mapping_rate]
warn = 85.0
fail = 70.0
direction = "HigherIsBetter"

[duplication_rate]
warn = 20.0
fail = 40.0
direction = "LowerIsBetter"

[error_rate]
warn = 0.01
fail = 0.02
direction = "LowerIsBetter"

[ts_tv_ratio]
warn = 1.5
fail = 1.2
direction = "HigherIsBetter"

[gc_deviation]
warn = 20.0
fail = 30.0
direction = "LowerIsBetter"
"#;
        let config: ThresholdConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.mapping_rate.warn, 85.0);
        assert_eq!(config.mapping_rate.fail, 70.0);
        assert_eq!(config.duplication_rate.warn, 20.0);
    }

    #[test]
    fn test_qc_level_worst() {
        assert_eq!(QcLevel::Pass.worst(QcLevel::Pass), QcLevel::Pass);
        assert_eq!(QcLevel::Pass.worst(QcLevel::Warn), QcLevel::Warn);
        assert_eq!(QcLevel::Warn.worst(QcLevel::Fail), QcLevel::Fail);
        assert_eq!(QcLevel::Fail.worst(QcLevel::Pass), QcLevel::Fail);
    }
}
