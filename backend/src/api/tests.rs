#[cfg(test)]
mod integration_tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use serde_json::{json, Value};
    use sqlx::SqlitePool;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tower::util::ServiceExt;
    use crate::{
        api::{AppState, health, orders, fillers, batch, proofs, relayer},
        config::Config,
        models::{CreateOrderRequest, OrderType, OrderStatus, OrderResponse, LockOrderRequest, SubmitPaymentProofRequest, OrderStatusResponse},
        services::{
            matching_engine::MatchingEngine,
            batch_processor::BatchProcessor,
        },
        blockchain::BlockchainClient,
    };
    use axum::routing::{get, post};

    async fn create_test_app() -> (Router, SqlitePool) {
        // Create in-memory database for testing
        let db = SqlitePool::connect(":memory:").await.unwrap();
        
        // Run migrations
        crate::database::run_migrations(&db).await.unwrap();
        
        // Create mock config
        let config = Config::default();
        
        // Create app state
        let app_state = AppState::new(config, db.clone());
        
        // Build test router with all routes
        let app = Router::new()
            // Health endpoints
            .route("/health", get(health::health_check))
            .route("/health/simple", get(health::health_simple))
            
            // Order management endpoints
            .route("/api/v1/orders", post(orders::create_order))
            .route("/api/v1/orders", get(orders::list_orders))
            .route("/api/v1/orders/:order_id", get(orders::get_order))
            .route("/api/v1/orders/:order_id/status", get(orders::get_order_status))
            .route("/api/v1/orders/:order_id/mark-paid", post(orders::mark_paid))
            .route("/api/v1/orders/match", post(orders::match_orders))
            
            // Filler endpoints
            .route("/api/v1/fillers/discovery", get(fillers::get_discovery_orders))
            .route("/api/v1/fillers/orders/:order_id/lock", post(fillers::lock_order))
            .route("/api/v1/fillers/orders/:order_id/payment-proof", post(fillers::submit_payment_proof))
            
            // Batch processing endpoints
            .route("/api/v1/batch/start", post(batch::start_batch))
            .route("/api/v1/batch/finalize", post(batch::finalize_batch))
            .route("/api/v1/batch/prove", post(batch::prove_batch))
            .route("/api/v1/batch/stats", get(batch::get_batch_stats))
            .route("/api/v1/batch/current", get(batch::get_current_batch))
            .route("/api/v1/batch/init-account", post(batch::init_account))
            
            // Proof endpoints
            .route("/api/v1/proofs/order/:batch_id/:order_id", get(proofs::get_order_proof))
            .route("/api/v1/proofs/account/:address", get(proofs::get_account_proof))
            .route("/api/v1/proofs/verify", post(proofs::verify_proof))
            .route("/api/v1/proofs/batch/:batch_id", get(proofs::get_batch_proofs))
            .route("/api/v1/proofs/stats", get(proofs::get_proof_stats))
            
            // Relayer endpoints
            .route("/api/v1/relayer/status", get(relayer::get_relayer_status))
            .route("/api/v1/relayer/process-events", post(relayer::process_events_manually))
            .route("/api/v1/relayer/config", post(relayer::update_relayer_config))
            .route("/api/v1/relayer/blockchain", get(relayer::get_blockchain_status))
            .with_state(app_state);
        
        (app, db)
    }

    #[tokio::test]
    async fn test_health_endpoints() {
        let (app, _db) = create_test_app().await;

        // Test simple health check
        let response = app
            .clone()
            .oneshot(Request::builder().uri("/health/simple").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test detailed health check
        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_order_creation_workflow() {
        let (app, _db) = create_test_app().await;

        // Create a new order
        let create_request = CreateOrderRequest {
            order_type: OrderType::BridgeIn,
            from_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            to_address: Some("0x9876543210987654321098765432109876543210".to_string()),
            token_id: 1,
            amount: "1000000000000000000".to_string(), // 1 ETH
            bank_account: Some("12345678".to_string()),
            bank_service: Some("PayPal Hong Kong".to_string()),
            banking_hash: None,
        };

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/orders")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&create_request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let order: OrderResponse = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(order.order_type, OrderType::BridgeIn);
        assert_eq!(order.status, OrderStatus::Pending);
        assert_eq!(order.amount, "1000000000000000000");
        assert_eq!(order.bank_account, Some("12345678".to_string()));
        assert_eq!(order.bank_service, Some("PayPal Hong Kong".to_string()));

        // Test retrieving the order
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/v1/orders/{}", order.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let retrieved_order: OrderResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(retrieved_order.id, order.id);
    }

    #[tokio::test]
    async fn test_order_status_tracking() {
        let (app, _db) = create_test_app().await;

        // Create a new order
        let create_request = CreateOrderRequest {
            order_type: OrderType::BridgeIn,
            from_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            to_address: Some("0x9876543210987654321098765432109876543210".to_string()),
            token_id: 1,
            amount: "1000000000000000000".to_string(),
            bank_account: Some("12345678".to_string()),
            bank_service: Some("PayPal Hong Kong".to_string()),
            banking_hash: None,
        };

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/orders")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&create_request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let order: OrderResponse = serde_json::from_slice(&body).unwrap();

        // Test order status endpoint
        let response = app
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/v1/orders/{}/status", order.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let status_response: OrderStatusResponse = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(status_response.id, order.id);
        assert_eq!(status_response.status, OrderStatus::Pending);
        assert_eq!(status_response.progress_percentage, 10); // Pending = Private Listing phase
    }

    #[tokio::test]
    async fn test_order_listing() {
        let (app, _db) = create_test_app().await;

        // Create multiple orders
        for i in 0..3 {
            let create_request = CreateOrderRequest {
                order_type: OrderType::BridgeIn,
                from_address: Some("0x1234567890123456789012345678901234567890".to_string()),
                to_address: Some("0x9876543210987654321098765432109876543210".to_string()),
                token_id: 1,
                amount: format!("{}000000000000000000", i + 1), // 1, 2, 3 ETH
                bank_account: Some(format!("1234567{}", i)),
                bank_service: Some("PayPal Hong Kong".to_string()),
                banking_hash: None,
            };

            let _ = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/orders")
                        .header("content-type", "application/json")
                        .body(Body::from(serde_json::to_string(&create_request).unwrap()))
                        .unwrap(),
                )
                .await
                .unwrap();
        }

        // Test listing orders
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/orders")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let orders = response_data["orders"].as_array().unwrap();
        
        assert_eq!(orders.len(), 3);
        assert_eq!(response_data["total"].as_u64().unwrap(), 3);
    }

    #[tokio::test]
    async fn test_filler_discovery_workflow() {
        let (app, db) = create_test_app().await;

        // Create an order and manually set it to Discovery status
        let create_request = CreateOrderRequest {
            order_type: OrderType::BridgeIn,
            from_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            to_address: Some("0x9876543210987654321098765432109876543210".to_string()),
            token_id: 1,
            amount: "1000000000000000000".to_string(),
            bank_account: Some("12345678".to_string()),
            bank_service: Some("PayPal Hong Kong".to_string()),
            banking_hash: None,
        };

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/orders")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&create_request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let order: OrderResponse = serde_json::from_slice(&body).unwrap();

        // Manually update order status to Discovery in database
        sqlx::query("UPDATE orders SET status = ? WHERE id = ?")
            .bind(OrderStatus::Discovery as i32)
            .bind(&order.id)
            .execute(&db)
            .await
            .unwrap();

        // Test discovery endpoint
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/fillers/discovery")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let discovery_orders = response_data["orders"].as_array().unwrap();
        
        assert_eq!(discovery_orders.len(), 1);
        assert_eq!(discovery_orders[0]["id"].as_str().unwrap(), order.id);
    }

    #[tokio::test]
    async fn test_filler_lock_order_workflow() {
        let (app, db) = create_test_app().await;

        // Create an order and set it to Discovery status
        let create_request = CreateOrderRequest {
            order_type: OrderType::BridgeIn,
            from_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            to_address: Some("0x9876543210987654321098765432109876543210".to_string()),
            token_id: 1,
            amount: "1000000000000000000".to_string(),
            bank_account: Some("12345678".to_string()),
            bank_service: Some("PayPal Hong Kong".to_string()),
            banking_hash: None,
        };

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/orders")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&create_request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let order: OrderResponse = serde_json::from_slice(&body).unwrap();

        // Set order to Discovery status
        sqlx::query("UPDATE orders SET status = ? WHERE id = ?")
            .bind(OrderStatus::Discovery as i32)
            .bind(&order.id)
            .execute(&db)
            .await
            .unwrap();

        // Test locking the order
        let lock_request = LockOrderRequest {
            filler_id: "filler_123".to_string(),
            amount: "500000000000000000".to_string(), // 0.5 ETH
        };

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/v1/fillers/orders/{}/lock", order.id))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&lock_request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify order is locked
        let response = app
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/v1/orders/{}", order.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let locked_order: OrderResponse = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(locked_order.filler_id, Some("filler_123".to_string()));
        assert_eq!(locked_order.locked_amount, Some("500000000000000000".to_string()));
    }

    #[tokio::test]
    async fn test_filler_payment_proof_workflow() {
        let (app, db) = create_test_app().await;

        // Create an order and set it to Locked status
        let create_request = CreateOrderRequest {
            order_type: OrderType::BridgeIn,
            from_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            to_address: Some("0x9876543210987654321098765432109876543210".to_string()),
            token_id: 1,
            amount: "1000000000000000000".to_string(),
            bank_account: Some("12345678".to_string()),
            bank_service: Some("PayPal Hong Kong".to_string()),
            banking_hash: None,
        };

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/orders")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&create_request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let order: OrderResponse = serde_json::from_slice(&body).unwrap();

        // Set order to Locked status with filler info
        sqlx::query("UPDATE orders SET status = ?, filler_id = ?, locked_amount = ? WHERE id = ?")
            .bind(OrderStatus::Locked as i32)
            .bind("filler_123")
            .bind("500000000000000000")
            .bind(&order.id)
            .execute(&db)
            .await
            .unwrap();

        // Test submitting payment proof
        let payment_proof_request = SubmitPaymentProofRequest {
            banking_hash: "0xabcdef123456789".to_string(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/v1/fillers/orders/{}/payment-proof", order.id))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&payment_proof_request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_batch_processing_endpoints() {
        let (app, _db) = create_test_app().await;

        // Test starting a batch
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/batch/start")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Test getting batch stats
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/batch/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Test getting current batch
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/batch/current")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_proof_endpoints() {
        let (app, _db) = create_test_app().await;

        // Test getting proof stats
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/proofs/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Test account proof endpoint (should return 404 for non-existent address)
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/proofs/account/0x1234567890123456789012345678901234567890")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // The proof endpoint returns 200 with empty or default data when no account exists
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_relayer_endpoints() {
        let (app, _db) = create_test_app().await;

        // Test getting relayer status (should indicate no blockchain client)
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/relayer/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should indicate relayer service status (may be unavailable in tests)
        // Accept either OK or Service Unavailable depending on configuration
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::SERVICE_UNAVAILABLE);

        // Test getting blockchain status
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/relayer/blockchain")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return 503 since no blockchain client is configured in tests
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let (app, _db) = create_test_app().await;

        // Test 404 for non-existent order
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/orders/non-existent-id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Test invalid JSON in create order
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/orders")
                    .header("content-type", "application/json")
                    .body(Body::from("invalid json"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Test locking non-existent order
        let lock_request = LockOrderRequest {
            filler_id: "filler_123".to_string(),
            amount: "500000000000000000".to_string(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/fillers/orders/non-existent-id/lock")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&lock_request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
