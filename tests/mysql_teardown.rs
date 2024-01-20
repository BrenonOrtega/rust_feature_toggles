use mysql::prelude::Queryable;

pub(crate) struct Teardown {
    pub(crate) conn_str: String,
    pub(crate) database: String,
}

impl Drop for Teardown {
    fn drop(&mut self) {
        use mysql::Pool;
        let pool = Pool::new(self.conn_str.as_str()).unwrap();
        let mut conn = pool.get_conn().unwrap();

        conn.query_drop(format!("DROP TABLE {}.feature_toggles;", self.database))
            .unwrap();
        println!("Tearing down.");
    }
}
