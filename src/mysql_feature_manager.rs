use std::sync::Arc;
use mysql::prelude::Queryable;
use mysql::PooledConn;
use crate::{feature_toggles::{FeatureToggle, FeatureState}, 
    feature_manager::FeatureManager};

#[cfg(feature = "mysql")]
struct MySqlFeatureManager {
    features: Vec<FeatureToggle>
}

#[cfg(feature = "mysql")]
impl MySqlFeatureManager {
    fn new(features: Vec<FeatureToggle>) -> Self {
        Self {
            features
        }
    }
}

#[cfg(feature = "mysql")]
impl FeatureManager for MySqlFeatureManager {
    fn resolve(&self, feature_name: &str) -> Option<Box<dyn FeatureState>> {
        let feature = self.features
                .iter()
                .find(|feature| feature.name() == feature_name)
                .cloned();

        match feature {
            Some(feature) => Some(Box::new(feature)),
            None => None,
        }
    }
}

#[cfg(feature = "mysql")]
pub enum TogglesStatus {
    HasAny(Arc<dyn FeatureManager>),
    Empty(Arc<dyn FeatureManager>),
    FailedInitialization,
}

#[cfg(feature = "mysql")]
pub fn use_mysql_feature_manager(connection_string: &str, features: Vec<FeatureToggle>) -> TogglesStatus {
    use_mysql_feature_manager_on("features_management", connection_string, features)
}

#[cfg(feature = "mysql")]
pub fn use_mysql_feature_manager_on(database: &str, connection_string: &str,  features: Vec<FeatureToggle>) -> TogglesStatus {
    use mysql::Pool;

    let pool = Pool::new(connection_string).unwrap();
    let conn_result = pool.get_conn();
    
    if let Err(err) = conn_result {
        println!("{:?}", err);
        return TogglesStatus::FailedInitialization;
    }

    let mut conn = conn_result.unwrap();

    conn.query_drop(format!(r#"
                CREATE DATABASE IF NOT EXISTS {};
                CREATE TABLE IF NOT EXISTS {}.feature_toggles (
                    name VARCHAR(100) PRIMARY KEY NOT NULL,
                    state TINYINT NOT NULL);"#, database, database)).unwrap();

    if let Err(e) = insert_features(database, &mut conn, features) {
        println!("{:?}", e);
        return TogglesStatus::FailedInitialization;   
    }

    load_feature_manager(database, conn)
}

#[cfg(feature = "mysql")]
fn insert_features(database: &str, conn: &mut PooledConn, features: Vec<FeatureToggle>) -> Result<(), mysql::Error> {
    if features.len() == 0 {
        return Ok(());
    }

    let insert = format!("INSERT IGNORE INTO {}.feature_toggles (name, state) VALUES", database);
    let statements = features.iter()
        .map(|feature| format!("(\'{}\', {})", feature.name, feature.state))
        .reduce(|initial, next| format!("{}, {}", initial, next))
        .unwrap();
    
    let sql = format!("{} {}", insert, statements);

    conn.query_drop(sql)
}

fn load_feature_manager(database: &str, mut conn: mysql::PooledConn) -> TogglesStatus {
    let result: Result<Vec<FeatureToggle>, mysql::Error> 
        = conn.query_map(format!("SELECT name, state FROM {}.feature_toggles;", database),
            |(name, state)| FeatureToggle::new(name, state));

    let result = match result {
        Ok(features) => {
            if features.len() > 0 {
                let my_sql_feature_manager: Arc<dyn FeatureManager> = Arc::new(MySqlFeatureManager::new(features));
                TogglesStatus::HasAny(my_sql_feature_manager)
            }
            else {
                TogglesStatus::Empty(Arc::new(MySqlFeatureManager::new(Vec::new())))
            }
        },
        Err(e) => {
            println!("{}", e);
            TogglesStatus::FailedInitialization
        }
    };

    result
}
