use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite:../tadpole.db").await?;
    
    let rows: Vec<(String, String, f64, f64)> = sqlx::query_as("SELECT id, entity_id, budget_usd, used_usd FROM agent_quotas")
        .fetch_all(&pool)
        .await?;

    println!("Found {} quotas in database:", rows.len());
    for row in rows {
        println!("ID: {}, Entity: {}, Budget: ${}, Used: ${}", row.0, row.1, row.2, row.3);
    }

    Ok(())
}
