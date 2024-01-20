pub trait FeatureState {
    fn enabled(&self) -> bool;
    fn disabled(&self) -> bool;
    fn name(&self) -> &str;
}

#[derive(Debug, PartialEq, Eq)]
pub struct FeatureToggle {
    pub name: String,
    pub(crate) state: bool,
}

impl FeatureToggle {
    pub fn new(name: String, state: bool) -> Self {
        FeatureToggle {
            name,
            state
        }
    }
}

impl Clone for FeatureToggle {
    fn clone(&self) -> Self {
        Self { name: self.name.clone(), state: self.state.clone() }
    }
}

impl FeatureState for FeatureToggle {
    fn enabled(&self) -> bool {
        self.state
    }

    fn disabled(&self) -> bool {
       !self.enabled()
    }

    fn name(&self) -> &str {
        &self.name
    }
}

mod tests {
    use crate::feature_toggles::{FeatureToggle, FeatureState};

    #[test]
    fn should_have_correct_boolean_value() {
        let feature_toggle = FeatureToggle::new("test_feature_toggle".to_string(), true);

        assert!(feature_toggle.enabled());
        assert!(feature_toggle.disabled() == false);
    }

    #[test]
    fn should_have_expected_name() {
        let expected_feature_name = "test_feature_toggle";
        let feature_toggle = FeatureToggle::new(expected_feature_name.to_string(), true);

        assert!(feature_toggle.name() == expected_feature_name);
    }
}