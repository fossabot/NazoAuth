use diesel_async::{
    AsyncPgConnection,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Object, deadpool::Pool},
};

pub(crate) type DbPool = Pool<AsyncPgConnection>;
pub(crate) type DbConnection = Object<AsyncPgConnection>;

pub(crate) fn create_pool(database_url: String, max_connections: usize) -> anyhow::Result<DbPool> {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Ok(Pool::builder(manager).max_size(max_connections).build()?)
}

pub(crate) async fn get_conn(pool: &DbPool) -> anyhow::Result<DbConnection> {
    Ok(pool.get().await?)
}
