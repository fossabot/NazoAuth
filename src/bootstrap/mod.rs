//! 应用启动入口。
// 负责组装配置、外部连接、共享状态和 Actix HTTP server。

mod cors;
mod routes;

use std::{env, net::SocketAddr, sync::Arc};

use actix_web::{App, HttpServer, web};
use fred::{
    interfaces::ClientLike,
    prelude::{Builder as ValkeyBuilder, Config as ValkeyConfig},
};

use crate::db::create_pool;
use crate::domain::{AppState, Settings};
use crate::support::{load_or_create_keyset, normalize_database_url};

pub(crate) async fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // 环境变量只在启动阶段读取，运行期通过 AppState 共享不可变配置。
    let database_url = normalize_database_url(
        &env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:postgres@127.0.0.1:5432/oauth".into()),
    );
    let valkey_url = env::var("VALKEY_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/0".into());

    // 数据库和 Valkey 客户端在 server factory 外创建，避免每个 worker 重复初始化。
    let diesel_db = create_pool(database_url.clone(), 32)?;
    let valkey = ValkeyBuilder::from_config(ValkeyConfig::from_url(&valkey_url)?).build()?;
    valkey.init().await?;

    let settings = Arc::new(Settings::from_env());
    tokio::fs::create_dir_all(&settings.avatar_storage_dir)
        .await
        .ok();
    let keyset = Arc::new(load_or_create_keyset(&settings).await?);

    let state = web::Data::new(AppState {
        diesel_db,
        valkey,
        settings,
        keyset,
    });

    let bind = env::var("BIND").unwrap_or_else(|_| "0.0.0.0:8000".into());
    let addr: SocketAddr = bind.parse()?;
    tracing::info!("nazo-oauth-server(actix-web) listening on {addr}");

    HttpServer::new(move || {
        App::new()
            .wrap(cors::build(&state.settings))
            .app_data(state.clone())
            .configure(routes::configure)
    })
    .bind(addr)?
    .run()
    .await?;
    Ok(())
}
