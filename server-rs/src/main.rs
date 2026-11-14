use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tracing_subscriber::EnvFilter;

use lnc_styles_gallery::config::{AppSettings, EnvSnapshot, RuntimeStore};
use lnc_styles_gallery::crypto::MasterKeys;
use lnc_styles_gallery::oidc::{LoginLimiter, OidcClient, OidcSettings};
use lnc_styles_gallery::server::{build_router, AppState};
use lnc_styles_gallery::store::StyleLibraryStore;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--healthcheck") {
        std::process::exit(run_healthcheck().await);
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("lnc_styles_gallery=info,tower_http=info")),
        )
        .init();

    let env = EnvSnapshot::from_process();
    let settings = AppSettings::from_map(&env);

    let (master_keys, key_source) = match MasterKeys::load_or_init(&settings.config_dir) {
        Ok(pair) => pair,
        Err(e) => {
            tracing::error!(error = %e, "Failed to initialize LNC_MASTER_KEY");
            std::process::exit(1);
        }
    };
    tracing::info!(source = ?key_source, "Cryptographic master subkeys initialized");

    let runtime = match RuntimeStore::load_or_init(&settings.config_dir, master_keys, &env) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            tracing::error!(error = %e, "Failed to load runtime config/keys store");
            std::process::exit(1);
        }
    };

    let oidc_settings = OidcSettings::from_map(&env, &settings.public_url);
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("reqwest rustls client");
    let oidc = Arc::new(OidcClient::new(oidc_settings, http_client));

    let corpus = Arc::new(StyleLibraryStore::new(settings.clone()));
    let initial_status = corpus.reindex();
    tracing::info!(
        styles = initial_status["styles_count"].as_u64().unwrap_or(0),
        templates = initial_status["templates_count"].as_u64().unwrap_or(0),
        components = initial_status["components_count"].as_u64().unwrap_or(0),
        port = settings.port,
        "Initial corpus index ready"
    );

    let bind_addr = format!("{}:{}", settings.host, settings.port);
    let state = Arc::new(AppState {
        settings,
        runtime,
        corpus,
        oidc,
        limiter: Arc::new(LoginLimiter::default()),
    });

    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| {
            tracing::error!(addr = %bind_addr, error = %e, "Failed to bind TCP listener");
            std::process::exit(1);
        });

    tracing::info!(addr = %bind_addr, "lnc-styles-gallery listening");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap_or_else(|e| {
        tracing::error!(error = %e, "HTTP server exited with error");
    });
}

async fn run_healthcheck() -> i32 {
    let env = EnvSnapshot::from_process();
    let port = env.u16_first(&["PORT", "STYLES_PORT"], 8000);
    let url = format!("http://127.0.0.1:{port}/health");
    let Ok(client) = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
    else {
        return 1;
    };
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => 0,
        _ => 1,
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            sig.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
