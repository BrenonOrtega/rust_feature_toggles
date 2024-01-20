mod constants;
mod mysql_teardown;
use crate::constants::TEST_DB;
use crate::mysql_teardown::Teardown;
use feature_toggles::{
    feature_manager::FeatureManager,
    feature_toggles::{FeatureState, FeatureToggle},
};
use std::sync::Arc;

fn get_default_teardown(conn_str: &str) -> Teardown {
    Teardown {
        conn_str: conn_str.to_string(),
        database: TEST_DB.to_string(),
    }
}

#[test]
#[cfg(feature = "mysql")]
fn should_create_table_correctly() {
    use feature_toggles::mysql_feature_manager::{use_mysql_feature_manager_on, TogglesStatus};
    let conn_str = "mysql://root:password@localhost:3307/db";
    let _teardown = get_default_teardown(conn_str);
    let feat = Vec::new();

    let feature_statuses: TogglesStatus = use_mysql_feature_manager_on(TEST_DB, conn_str, feat);

    match feature_statuses {
        TogglesStatus::Empty(_) => assert!(true, "Database does not have any feature flag"),
        _ => assert!(false, "Test failed"),
    }
}

#[test]
#[cfg(feature = "mysql")]
fn shouldnt_panic_if_called_more_than_once() {
    use feature_toggles::mysql_feature_manager::{use_mysql_feature_manager_on, TogglesStatus};

    let conn_str = "mysql://root:password@localhost:3307/db";
    let _teardown = get_default_teardown(conn_str);

    let feat = Vec::new();
    let _feature_statuses = use_mysql_feature_manager_on(TEST_DB, conn_str, feat.clone());
    let feature_statuses = use_mysql_feature_manager_on(TEST_DB, conn_str, feat);

    match feature_statuses {
        TogglesStatus::FailedInitialization => assert!(false, "Panicked for being called 2 times."),
        _ => assert!(true, "Test Passed"),
    }
}

#[test]
#[cfg(feature = "mysql")]
fn loading_feature_toggles_should_work() {
    use feature_toggles::{
        feature_toggles::FeatureToggle,
        mysql_feature_manager::{use_mysql_feature_manager_on, TogglesStatus},
    };

    let conn_str = "mysql://root:password@localhost:3307/db";
    let _teardown = get_default_teardown(conn_str);

    let features = vec![
        FeatureToggle::new("TOGGLE_TEST_FEATURE".to_string(), true),
        FeatureToggle::new("OTHER_TEST_FEATURE".to_string(), false),
        FeatureToggle::new("YET_OTHER_TEST_FEATURE".to_string(), true),
    ];

    let feature_statuses = use_mysql_feature_manager_on(TEST_DB, conn_str, features.clone());

    match feature_statuses {
        TogglesStatus::HasAny(manager) => {
            assert_features_are_loaded(manager, features, |feature, actual_feature| {
                assert_eq!(
                    feature.enabled(),
                    actual_feature.enabled(),
                    "Newly loaded feature should represent saved feature."
                )
            })
        }
        _ => assert!(false, "TEST FAILED WHEN BUILDING FEATURE MANAGER"),
    }
}

fn assert_features_are_loaded(
    feature_manager: Arc<dyn FeatureManager>,
    features: Vec<FeatureToggle>,
    assert_function: fn(&FeatureToggle, Box<dyn FeatureState>),
) {
    features
        .iter()
        .for_each(|feature| match feature_manager.resolve(&feature.name) {
            Some(actual_feature) => assert_function(feature, actual_feature),
            _ => assert!(
                false,
                "Features are not being inserted correctly in the DB."
            ),
        });
}

#[test]
#[cfg(feature = "mysql")]
fn loading_duplicated_features_should_not_panic() {
    use feature_toggles::{
        feature_toggles::FeatureToggle,
        mysql_feature_manager::{use_mysql_feature_manager_on, TogglesStatus},
    };

    let conn_str = "mysql://root:password@localhost:3307/db";
    let _teardown = get_default_teardown(conn_str);

    let feature_name = "TOGGLE_TEST_FEATURE".to_string();
    let features = vec![
        FeatureToggle::new(feature_name.clone(), true),
        FeatureToggle::new(feature_name, false),
    ];

    let feature_statuses: TogglesStatus =
        use_mysql_feature_manager_on(TEST_DB, conn_str, features.clone());

    match feature_statuses {
        TogglesStatus::HasAny(manager) => match manager.resolve(features[0].name()) {
            Some(actual_feature) => assert_eq!(features[0].enabled(), actual_feature.enabled()),

            _ => panic!("TEST FAILED"),
        },
        _ => assert!(false, "TEST FAILED WHEN BUILDING FEATURE MANAGER"),
    }
}
