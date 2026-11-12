use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Plan {
    Free,
    Pro,
}

impl Plan {
    pub fn from_setting(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "pro" => Plan::Pro,
            _ => Plan::Free,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Plan::Free => "free",
            Plan::Pro => "pro",
        }
    }

    pub fn can(self, feature: &str) -> bool {
        match feature {
            "monthly" | "goal" | "second_reminder" | "progress_deep" | "unlimited_tasks" => {
                self == Plan::Pro
            }
            "overlay" | "reminders" | "quick_add" | "daily" | "weekly" => true,
            _ => self == Plan::Pro,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_never_paywalls_overlay() {
        assert!(Plan::Free.can("overlay"));
        assert!(Plan::Free.can("reminders"));
        assert!(!Plan::Free.can("monthly"));
        assert!(Plan::Pro.can("monthly"));
    }
}
