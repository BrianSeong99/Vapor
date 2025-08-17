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

    // Initialize blockchain client (for MVP, we'll create a simple mock)
    // In production, you'd initialize this with real contract addresses
    // let blockchain_client = blockchain::BlockchainClient::new(...).await?;

    // For MVP, we'll initialize the app state without blockchain client
    // The relayer will be disabled until blockchain client is properly configured
    let app_state = api::AppState::new(config, db);

    // TODO: Add blockchain client initialization when contract addresses are available
    // app_state = app_state.with_blockchain_client(blockchain_client);

    // TODO: Initialize and start relayer service
    // if let Some(blockchain_client) = &app_state.blockchain_client {
    //     let relayer_config = services::relayer::RelayerConfig::default();
    //     let relayer = services::relayer::RelayerService::new(
    //         blockchain_client.clone(),
    //         app_state.db.clone(),
    //         app_state.matching_engine.clone(),
    //         app_state.batch_processor.clone(),
    //         relayer_config.clone(),
    //     ).await?;
    //     
    //     app_state = app_state.with_relayer_service(relayer).await;
    //     
    //     // Start relayer service in background
    //     let relayer_service = app_state.relayer_service.clone();
    //     tokio::spawn(async move {
    //         if let Some(relayer_service) = relayer_service {
    //             if let Ok(mut relayer) = relayer_service.try_lock() {
    //                 if let Err(e) = relayer.start(relayer_config).await {
    //                     error!("Relayer service failed: {}", e);
    //                 }
    //             }
    //         }
    //     });
    //     
    //     info!("Relayer service started in background");
    // } else {
    //     warn!("Blockchain client not configured, relayer service disabled");
    // }

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
        
        // Relayer endpoints
        .route("/api/v1/relayer/status", get(api::relayer::get_relayer_status))
        .route("/api/v1/relayer/process-events", post(api::relayer::process_events_manually))
        .route("/api/v1/relayer/config", post(api::relayer::update_relayer_config))
        .route("/api/v1/relayer/blockchain", get(api::relayer::get_blockchain_status))
        
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
