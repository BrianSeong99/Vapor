use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod api;
mod config;
mod database;
mod models;
mod services;
mod blockchain;
mod merkle;

// Library modules
mod lib {
    pub mod sparse_merkle_tree;
    
    pub use sparse_merkle_tree::{
        SparseMerkleTree, 
        SparseMerkleLeaf, 
        MerkleProof, 
        ethereum_address_to_path, 
        index_to_path
    };
}

use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Load configuration
    dotenv::dotenv().ok();
    let config = Config::from_env()?;
    
    info!("Starting Cashlink Backend Server...");
    info!("Contract address: {}", config.blockchain.contract_address);

    // Initialize database
    let db = database::init_db(&config.database.url).await?;
    
    // Run database migrations
    database::run_migrations(&db).await?;

    // Store port before moving config
    let port = config.api.port;

    // Build our application with routes
    let app = Router::new()
        // Health endpoints
        .route("/health", get(api::health::health_check))
        .route("/health/simple", get(api::health::health_simple))
        
        // Order management endpoints
        .route("/api/v1/orders", post(api::orders::create_order))
        .route("/api/v1/orders", get(api::orders::list_orders))
        .route("/api/v1/orders/:order_id", get(api::orders::get_order))
        .route("/api/v1/orders/:order_id/mark-paid", post(api::orders::mark_paid))
        .route("/api/v1/orders/match", post(api::orders::match_orders))
        
        // Batch processing endpoints
        .route("/api/v1/batch/start", post(api::batch::start_batch))
        .route("/api/v1/batch/finalize", post(api::batch::finalize_batch))
        .route("/api/v1/batch/prove", post(api::batch::prove_batch))
        .route("/api/v1/batch/stats", get(api::batch::get_batch_stats))
        .route("/api/v1/batch/current", get(api::batch::get_current_batch))
        .route("/api/v1/batch/init-account", post(api::batch::init_account))
        
        // Proof endpoints
        .route("/api/v1/proofs/order/:batch_id/:order_id", get(api::proofs::get_order_proof))
        .route("/api/v1/proofs/account/:address", get(api::proofs::get_account_proof))
        .route("/api/v1/proofs/verify", post(api::proofs::verify_proof))
        .route("/api/v1/proofs/batch/:batch_id", get(api::proofs::get_batch_proofs))
        .route("/api/v1/proofs/stats", get(api::proofs::get_proof_stats))
        
        .layer(CorsLayer::permissive())
        .with_state(api::AppState::new(config, db));

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
