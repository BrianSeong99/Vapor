use sqlx::{sqlite::SqlitePoolOptions, SqlitePool, Row};
use tracing::info;
use anyhow::Result;

pub async fn init_db(database_url: &str) -> Result<SqlitePool> {
    info!("Connecting to database: {}", database_url);
    
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    
    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    info!("Running database migrations...");
    
    // Create orders table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS orders (
            id TEXT PRIMARY KEY,
            order_type INTEGER NOT NULL,
            from_address TEXT,
            to_address TEXT, 
            token_id INTEGER NOT NULL,
            amount TEXT NOT NULL,
            bank_account TEXT,
            bank_service TEXT,
            banking_hash TEXT,
            filler_id TEXT,
            locked_amount TEXT,
            status INTEGER NOT NULL DEFAULT 0,
            batch_id INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create batches table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS batches (
            id INTEGER PRIMARY KEY,
            prev_state_root TEXT NOT NULL,
            prev_orders_root TEXT NOT NULL,
            new_state_root TEXT NOT NULL,
            new_orders_root TEXT NOT NULL,
            proof_data TEXT,
            status INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            submitted_at DATETIME
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create account_balances table (fixed schema for multi-token support)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS account_balances (
            id INTEGER PRIMARY KEY,
            address TEXT NOT NULL,
            token_id INTEGER NOT NULL,
            balance TEXT NOT NULL DEFAULT '0',
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(address, token_id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create filler_balances table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS filler_balances (
            filler_id TEXT PRIMARY KEY,
            total_balance TEXT NOT NULL DEFAULT '0',
            locked_balance TEXT NOT NULL DEFAULT '0',
            completed_jobs INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create filler_wallets table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS filler_wallets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filler_id TEXT NOT NULL,
            wallet_address TEXT NOT NULL,
            balance TEXT NOT NULL DEFAULT '0',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (filler_id) REFERENCES filler_balances(filler_id),
            UNIQUE(filler_id, wallet_address)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create claims table to track claim history
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS claims (
            id TEXT PRIMARY KEY,
            filler_id TEXT NOT NULL,
            wallet_address TEXT NOT NULL,
            destination_address TEXT NOT NULL,
            amount TEXT NOT NULL,
            batch_id INTEGER,
            transaction_hash TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            merkle_proof TEXT, -- JSON array of proof elements
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (filler_id) REFERENCES filler_balances(filler_id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    info!("Database migrations completed");
    Ok(())
}

/// Database helper functions for testing and operations
pub mod helpers {
    use super::*;
    use chrono::Utc;
    use crate::models::{Order, OrderType, OrderStatus, TokenBalance, FillerBalance, FillerWallet, ClaimRequest, ClaimResponse};
    
    /// Insert an order into the database
    pub async fn insert_order(pool: &SqlitePool, order: &Order) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO orders (id, order_type, status, from_address, to_address, token_id, amount, bank_account, bank_service, banking_hash, filler_id, locked_amount, batch_id, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            "#,
        )
        .bind(&order.id)
        .bind(order.order_type as i32)
        .bind(order.status as i32)
        .bind(&order.from_address)
        .bind(&order.to_address)
        .bind(order.token_id as i32)
        .bind(&order.amount)
        .bind(&order.bank_account)
        .bind(&order.bank_service)
        .bind(&order.banking_hash)
        .bind(&order.filler_id)
        .bind(&order.locked_amount)
        .bind(order.batch_id.map(|id| id as i32))
        .bind(order.created_at)
        .bind(order.updated_at)
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    /// Get an order by ID
    pub async fn get_order_by_id(pool: &SqlitePool, order_id: &str) -> Result<Option<Order>> {
        let row = sqlx::query(
            "SELECT id, order_type, status, from_address, to_address, token_id, amount, bank_account, bank_service, banking_hash, filler_id, locked_amount, batch_id, created_at, updated_at FROM orders WHERE id = ?"
        )
        .bind(order_id)
        .fetch_optional(pool)
        .await?;
        
        if let Some(row) = row {
            let order = Order {
                id: row.try_get("id")?,
                order_type: match row.try_get::<i32, _>("order_type")? {
                    0 => OrderType::BridgeIn,
                    1 => OrderType::BridgeOut,
                    2 => OrderType::Transfer,
                    _ => return Err(anyhow::anyhow!("Invalid order type")),
                },
                status: match row.try_get::<i32, _>("status")? {
                    0 => OrderStatus::Pending,
                    1 => OrderStatus::Discovery,
                    2 => OrderStatus::Locked,
                    3 => OrderStatus::MarkPaid,
                    4 => OrderStatus::Settled,
                    5 => OrderStatus::Failed,
                    _ => return Err(anyhow::anyhow!("Invalid order status")),
                },
                from_address: row.try_get("from_address")?,
                to_address: row.try_get("to_address")?,
                token_id: row.try_get::<i32, _>("token_id")? as u32,
                amount: row.try_get("amount")?,
                bank_account: row.try_get("bank_account")?,
                bank_service: row.try_get("bank_service")?,
                banking_hash: row.try_get("banking_hash")?,
                filler_id: row.try_get("filler_id")?,
                locked_amount: row.try_get("locked_amount")?,
                batch_id: row.try_get::<Option<i32>, _>("batch_id")?.map(|id| id as u32),
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            };
            Ok(Some(order))
        } else {
            Ok(None)
        }
    }
    
    /// Update account balance
    pub async fn upsert_account_balance(
        pool: &SqlitePool, 
        address: &str, 
        token_id: u32, 
        balance: &str
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO account_balances (address, token_id, balance, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(address, token_id) 
            DO UPDATE SET balance = ?3, updated_at = ?4
            "#,
        )
        .bind(address)
        .bind(token_id as i32)
        .bind(balance)
        .bind(Utc::now())
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    /// Get account balances for an address
    pub async fn get_account_balances(pool: &SqlitePool, address: &str) -> Result<Vec<TokenBalance>> {
        let rows = sqlx::query(
            "SELECT token_id, balance FROM account_balances WHERE address = ?"
        )
        .bind(address)
        .fetch_all(pool)
        .await?;
        
        let mut balances = Vec::new();
        for row in rows {
            balances.push(TokenBalance {
                token_id: row.try_get::<i32, _>("token_id")? as u32,
                balance: row.try_get("balance")?,
            });
        }
        
        Ok(balances)
    }
    
    /// Count orders by status
    pub async fn count_orders_by_status(pool: &SqlitePool, status: OrderStatus) -> Result<i64> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM orders WHERE status = ?"
        )
        .bind(status as i32)
        .fetch_one(pool)
        .await?;
        
        Ok(row.try_get("count")?)
    }
    
    /// Clean up all test data
    pub async fn cleanup_test_data(pool: &SqlitePool) -> Result<()> {
        sqlx::query("DELETE FROM orders").execute(pool).await?;
        sqlx::query("DELETE FROM batches").execute(pool).await?;
        sqlx::query("DELETE FROM account_balances").execute(pool).await?;
        Ok(())
    }

    /// Get filler balance by filler ID
    pub async fn get_filler_balance(pool: &SqlitePool, filler_id: &str) -> Result<Option<FillerBalance>> {
        // Get main balance info
        let balance_row = sqlx::query(
            "SELECT filler_id, total_balance, locked_balance, completed_jobs FROM filler_balances WHERE filler_id = ?"
        )
        .bind(filler_id)
        .fetch_optional(pool)
        .await?;

        let Some(row) = balance_row else {
            return Ok(None);
        };

        let total_balance: String = row.try_get("total_balance")?;
        let locked_balance: String = row.try_get("locked_balance")?;
        let completed_jobs: i32 = row.try_get("completed_jobs")?;

        // Calculate available balance
        let total: u64 = total_balance.parse().unwrap_or(0);
        let locked: u64 = locked_balance.parse().unwrap_or(0);
        let available_balance = (total.saturating_sub(locked)).to_string();

        // Get wallets for this filler
        let wallet_rows = sqlx::query(
            "SELECT wallet_address, balance FROM filler_wallets WHERE filler_id = ?"
        )
        .bind(filler_id)
        .fetch_all(pool)
        .await?;

        let wallets: Vec<FillerWallet> = wallet_rows
            .iter()
            .map(|row| {
                let balance: String = row.try_get("balance").unwrap_or_default();
                let wallet_balance: u64 = balance.parse().unwrap_or(0);
                let percentage = if total > 0 {
                    (wallet_balance as f32 / total as f32) * 100.0
                } else {
                    0.0
                };

                FillerWallet {
                    address: row.try_get("wallet_address").unwrap_or_default(),
                    balance,
                    percentage,
                }
            })
            .collect();

        Ok(Some(FillerBalance {
            filler_id: filler_id.to_string(),
            total_balance,
            available_balance,
            locked_balance,
            completed_jobs: completed_jobs as u32,
            wallets,
        }))
    }

    /// Create or update filler balance
    pub async fn upsert_filler_balance(pool: &SqlitePool, filler_id: &str, total_balance: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO filler_balances (filler_id, total_balance) 
            VALUES (?, ?)
            ON CONFLICT(filler_id) 
            DO UPDATE SET 
                total_balance = excluded.total_balance,
                updated_at = CURRENT_TIMESTAMP
            "#
        )
        .bind(filler_id)
        .bind(total_balance)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Add wallet to filler
    pub async fn add_filler_wallet(pool: &SqlitePool, filler_id: &str, wallet_address: &str, balance: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO filler_wallets (filler_id, wallet_address, balance) 
            VALUES (?, ?, ?)
            ON CONFLICT(filler_id, wallet_address) 
            DO UPDATE SET 
                balance = excluded.balance,
                updated_at = CURRENT_TIMESTAMP
            "#
        )
        .bind(filler_id)
        .bind(wallet_address)
        .bind(balance)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update filler locked balance
    pub async fn update_filler_locked_balance(pool: &SqlitePool, filler_id: &str, locked_balance: &str) -> Result<()> {
        sqlx::query(
            "UPDATE filler_balances SET locked_balance = ?, updated_at = CURRENT_TIMESTAMP WHERE filler_id = ?"
        )
        .bind(locked_balance)
        .bind(filler_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Record a claim
    pub async fn insert_claim(pool: &SqlitePool, claim_id: &str, filler_id: &str, wallet_address: &str, 
                            destination_address: &str, amount: &str, batch_id: Option<u32>) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO claims (id, filler_id, wallet_address, destination_address, amount, batch_id) 
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(claim_id)
        .bind(filler_id)
        .bind(wallet_address)
        .bind(destination_address)
        .bind(amount)
        .bind(batch_id.map(|id| id as i32))
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::helpers::*;
    use crate::models::{Order, OrderType, OrderStatus, TokenBalance};
    use chrono::Utc;
    use uuid::Uuid;

    async fn setup_test_db() -> SqlitePool {
        let pool = init_db("sqlite::memory:").await.expect("Failed to create test database");
        run_migrations(&pool).await.expect("Failed to run migrations");
        pool
    }
    
    fn create_test_order(id: &str, order_type: OrderType, status: OrderStatus, amount: &str) -> Order {
        Order {
            id: id.to_string(),
            order_type,
            status,
            from_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            to_address: Some("0x0987654321098765432109876543210987654321".to_string()),
            token_id: 1,
            amount: amount.to_string(),
            bank_account: Some("12345678".to_string()),
            bank_service: Some("PayPal Hong Kong".to_string()),
            banking_hash: Some("0xabcdef".to_string()),
            filler_id: None,
            locked_amount: None,
            batch_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_database_initialization() {
        let pool = setup_test_db().await;
        
        // Verify tables exist by trying to query them
        let result = sqlx::query("SELECT COUNT(*) FROM orders").fetch_one(&pool).await;
        assert!(result.is_ok(), "Orders table should exist");
        
        let result = sqlx::query("SELECT COUNT(*) FROM batches").fetch_one(&pool).await;
        assert!(result.is_ok(), "Batches table should exist");
        
        let result = sqlx::query("SELECT COUNT(*) FROM account_balances").fetch_one(&pool).await;
        assert!(result.is_ok(), "Account_balances table should exist");
    }

    #[tokio::test]
    async fn test_migrations_are_idempotent() {
        let pool = setup_test_db().await;
        
        // Run migrations again - should not fail
        let result = run_migrations(&pool).await;
        assert!(result.is_ok(), "Migrations should be idempotent");
    }

    #[tokio::test]
    async fn test_order_crud_operations() {
        let pool = setup_test_db().await;
        
        // Create test order
        let order = Order {
            id: Uuid::new_v4().to_string(),
            order_type: OrderType::BridgeIn,
            status: OrderStatus::Pending,
            from_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            to_address: Some("0x0987654321098765432109876543210987654321".to_string()),
            token_id: 1,
            amount: "1000000".to_string(), // 1 USDC
            bank_account: Some("12345678".to_string()),
            bank_service: Some("PayPal Hong Kong".to_string()),
            banking_hash: Some("0xabcdef".to_string()),
            filler_id: None,
            locked_amount: None,
            batch_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Insert order
        let result = insert_order(&pool, &order).await;
        assert!(result.is_ok(), "Should insert order successfully");
        
        // Retrieve order
        let retrieved = get_order_by_id(&pool, &order.id).await.unwrap();
        assert!(retrieved.is_some(), "Should retrieve inserted order");
        
        let retrieved_order = retrieved.unwrap();
        assert_eq!(retrieved_order.id, order.id);
        assert_eq!(retrieved_order.order_type, order.order_type);
        assert_eq!(retrieved_order.status, order.status);
        assert_eq!(retrieved_order.amount, order.amount);
        assert_eq!(retrieved_order.token_id, order.token_id);
    }

    #[tokio::test]
    async fn test_order_status_mapping() {
        let pool = setup_test_db().await;
        
        let test_cases = vec![
            OrderStatus::Pending,
            OrderStatus::Locked,
            OrderStatus::MarkPaid,
            OrderStatus::Settled,
            OrderStatus::Failed,
        ];
        
        for status in test_cases {
            let order = Order {
                id: Uuid::new_v4().to_string(),
                order_type: OrderType::BridgeIn,
                status: status.clone(),
                from_address: Some("0x1234567890123456789012345678901234567890".to_string()),
                to_address: Some("0x0987654321098765432109876543210987654321".to_string()),
                token_id: 1,
                amount: "1000000".to_string(),
                bank_account: Some("12345678".to_string()),
                bank_service: Some("PayPal Hong Kong".to_string()),
                banking_hash: Some("0xabcdef".to_string()),
                filler_id: None,
                locked_amount: None,
                batch_id: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            
            insert_order(&pool, &order).await.unwrap();
            let retrieved = get_order_by_id(&pool, &order.id).await.unwrap().unwrap();
            assert_eq!(retrieved.status, status, "Status should round-trip correctly");
        }
    }

    #[tokio::test]
    async fn test_account_balance_operations() {
        let pool = setup_test_db().await;
        
        let address = "0x1234567890123456789012345678901234567890";
        let token_id_usdc = 1;
        let token_id_pyusd = 2;
        
        // Insert initial balances
        upsert_account_balance(&pool, address, token_id_usdc, "1000000").await.unwrap();
        upsert_account_balance(&pool, address, token_id_pyusd, "500000").await.unwrap();
        
        // Retrieve balances
        let balances = get_account_balances(&pool, address).await.unwrap();
        assert_eq!(balances.len(), 2, "Should have 2 token balances");
        
        // Check specific balances
        let usdc_balance = balances.iter().find(|b| b.token_id == token_id_usdc).unwrap();
        assert_eq!(usdc_balance.balance, "1000000");
        
        let pyusd_balance = balances.iter().find(|b| b.token_id == token_id_pyusd).unwrap();
        assert_eq!(pyusd_balance.balance, "500000");
        
        // Update existing balance
        upsert_account_balance(&pool, address, token_id_usdc, "2000000").await.unwrap();
        
        let updated_balances = get_account_balances(&pool, address).await.unwrap();
        let updated_usdc = updated_balances.iter().find(|b| b.token_id == token_id_usdc).unwrap();
        assert_eq!(updated_usdc.balance, "2000000", "Balance should be updated");
    }

    #[tokio::test]
    async fn test_multi_token_constraint() {
        let pool = setup_test_db().await;
        
        let address = "0x1234567890123456789012345678901234567890";
        let token_id = 1;
        
        // Insert balance
        upsert_account_balance(&pool, address, token_id, "1000000").await.unwrap();
        
        // Try to insert duplicate (should update, not fail)
        let result = upsert_account_balance(&pool, address, token_id, "2000000").await;
        assert!(result.is_ok(), "Should handle duplicate address+token_id");
        
        // Verify only one record exists
        let balances = get_account_balances(&pool, address).await.unwrap();
        assert_eq!(balances.len(), 1, "Should have only one balance record per token");
        assert_eq!(balances[0].balance, "2000000", "Should have updated balance");
    }

    #[tokio::test]
    async fn test_order_counting() {
        let pool = setup_test_db().await;
        
        // Insert orders with different statuses
        for i in 0..3 {
            let mut order = create_test_order(
                &Uuid::new_v4().to_string(),
                OrderType::BridgeIn,
                OrderStatus::Pending,
                "1000000"
            );
            order.from_address = Some(format!("0x{:040x}", i));
            order.to_address = None;
            order.banking_hash = None;
            insert_order(&pool, &order).await.unwrap();
        }
        
        for i in 0..2 {
            let mut order = create_test_order(
                &Uuid::new_v4().to_string(),
                OrderType::BridgeIn,
                OrderStatus::Locked,
                "1000000"
            );
            order.from_address = Some(format!("0x{:040x}", i + 10));
            order.to_address = None;
            order.banking_hash = None;
            insert_order(&pool, &order).await.unwrap();
        }
        
        // Count orders by status
        let pending_count = count_orders_by_status(&pool, OrderStatus::Pending).await.unwrap();
        let locked_count = count_orders_by_status(&pool, OrderStatus::Locked).await.unwrap();
        let settled_count = count_orders_by_status(&pool, OrderStatus::Settled).await.unwrap();
        
        assert_eq!(pending_count, 3, "Should have 3 pending orders");
        assert_eq!(locked_count, 2, "Should have 2 locked orders");
        assert_eq!(settled_count, 0, "Should have 0 settled orders");
    }

    #[tokio::test]
    async fn test_cleanup_operations() {
        let pool = setup_test_db().await;
        
        // Insert test data
        let mut order = create_test_order(
            &Uuid::new_v4().to_string(),
            OrderType::BridgeIn,
            OrderStatus::Pending,
            "1000000"
        );
        order.from_address = Some("0x1234567890123456789012345678901234567890".to_string());
        order.to_address = None;
        order.banking_hash = None;
        insert_order(&pool, &order).await.unwrap();
        
        upsert_account_balance(&pool, "0x1234567890123456789012345678901234567890", 1, "1000000").await.unwrap();
        
        // Verify data exists
        let order_count = count_orders_by_status(&pool, OrderStatus::Pending).await.unwrap();
        let balances = get_account_balances(&pool, "0x1234567890123456789012345678901234567890").await.unwrap();
        
        assert_eq!(order_count, 1, "Should have test order");
        assert_eq!(balances.len(), 1, "Should have test balance");
        
        // Clean up
        cleanup_test_data(&pool).await.unwrap();
        
        // Verify data is gone
        let order_count_after = count_orders_by_status(&pool, OrderStatus::Pending).await.unwrap();
        let balances_after = get_account_balances(&pool, "0x1234567890123456789012345678901234567890").await.unwrap();
        
        assert_eq!(order_count_after, 0, "Should have no orders after cleanup");
        assert_eq!(balances_after.len(), 0, "Should have no balances after cleanup");
    }

    #[tokio::test]
    async fn test_large_amounts() {
        let pool = setup_test_db().await;
        
        // Test with very large amounts (18 decimal precision)
        let large_amount = "999999999999999999999999999999999999"; // 36 digits
        
        let mut order = create_test_order(
            &Uuid::new_v4().to_string(),
            OrderType::BridgeIn,
            OrderStatus::Pending,
            &large_amount.to_string()
        );
        order.from_address = Some("0x1234567890123456789012345678901234567890".to_string());
        order.to_address = None;
        order.banking_hash = None;
        
        // Should handle large amounts as strings
        insert_order(&pool, &order).await.unwrap();
        let retrieved = get_order_by_id(&pool, &order.id).await.unwrap().unwrap();
        assert_eq!(retrieved.amount, large_amount, "Should preserve large amount precision");
        
        // Test account balance with large amount
        upsert_account_balance(&pool, "0x1234567890123456789012345678901234567890", 1, large_amount).await.unwrap();
        let balances = get_account_balances(&pool, "0x1234567890123456789012345678901234567890").await.unwrap();
        assert_eq!(balances[0].balance, large_amount, "Should preserve large balance precision");
    }


}