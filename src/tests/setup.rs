use diesel::connection::SimpleConnection;
use diesel::r2d2::Pool;
use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
};
use once_cell::sync::Lazy;
use std::sync::Arc;
use testcontainers_modules::{
    mysql,
    testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner},
};

static MYSQL_CONTAINER: Lazy<tokio::sync::OnceCell<Arc<ContainerAsync<mysql::Mysql>>>> =
    Lazy::new(|| tokio::sync::OnceCell::new());

/// Creates a fresh, isolated database for a single test
pub async fn get_isolated_test_db() -> r2d2::PooledConnection<ConnectionManager<MysqlConnection>> {
    let container = MYSQL_CONTAINER
        .get_or_init(|| async {
            let image = mysql::Mysql::default()
                .with_env_var("MYSQL_ALLOW_EMPTY_PASSWORD", "yes")
                .with_env_var("MYSQL_DATABASE", "test");

            Arc::new(
                image
                    .start()
                    .await
                    .expect("Failed to start MySQL container"),
            )
        })
        .await
        .clone();

    // Get the actual host port
    let host_port = container.get_host_port_ipv4(3306).await.unwrap();

    let default_db_url = format!("mysql://root:@127.0.0.1:{host_port}/test");
    let manager = ConnectionManager::<MysqlConnection>::new(&default_db_url);
    let pool = Pool::builder().max_size(2).build(manager).unwrap();
    let mut conn = pool.get().unwrap();

    // Create an isolated database
    let db_name = format!("test_{}", uuid::Uuid::new_v4().simple());
    conn.batch_execute(&format!("CREATE DATABASE `{}`;", db_name))
        .unwrap();

    // Connect to the new isolated database
    let db_url = format!("mysql://root:@127.0.0.1:{host_port}/{}", db_name);
    let manager = ConnectionManager::<MysqlConnection>::new(&db_url);
    let pool = Pool::builder().max_size(2).build(manager).unwrap();
    let mut test_conn = pool.get().unwrap();

    create_tables(&mut test_conn).unwrap();

    test_conn
}

/// Create and initialize test data
pub fn create_tables(conn: &mut MysqlConnection) -> Result<(), diesel::result::Error> {
    conn.batch_execute(
        r#"
        CREATE TABLE IF NOT EXISTS bank (
            id CHAR(1) PRIMARY KEY,
            doints_on_hand DECIMAL(20,0) NOT NULL,
            total_doints DECIMAL(20,0) NOT NULL,
            tax_rate SMALLINT NOT NULL,
            ubi_rate SMALLINT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS fees (
            id CHAR(1) PRIMARY KEY,
            flat_fee DECIMAL(20,0) NOT NULL,
            percentage_fee SMALLINT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS users (
            id BIGINT UNSIGNED PRIMARY KEY,
            bal DECIMAL(20,0) NOT NULL
        );

        CREATE TABLE IF NOT EXISTS jail (
            id BIGINT UNSIGNED PRIMARY KEY,
            until TIMESTAMP NOT NULL,
            reason TINYTEXT NOT NULL,
            cause TINYTEXT NOT NULL,
            can_bail BOOL NOT NULL,
            CONSTRAINT fk_jail_user FOREIGN KEY (id) REFERENCES users(id)
        );

        -- Insert a default bank row if it doesn't exist
        INSERT INTO bank (id, doints_on_hand, total_doints, tax_rate, ubi_rate)
        SELECT 'B', 0, 1000000, 100, 0
        WHERE NOT EXISTS (SELECT 1 FROM bank);

        -- Insert a default fees row if it doesn't exist
        INSERT INTO fees (id, flat_fee, percentage_fee)
        SELECT 'F', 1, 10
        WHERE NOT EXISTS (SELECT 1 FROM fees);
    "#,
    )?;
    Ok(())
}
