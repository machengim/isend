use sqlx::Cursor;
use sqlx::Row;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "postgres://postgres:GJCKu5lMFLmLIvCy@10.214.144.3:5432/test";
    let pool = sqlx::PgPool::new(&url).await?;

    let mut cursor = sqlx::query("select * from customer").fetch(&pool);

    while let Some(row) = cursor.next().await? {
        let email: &str = row.get("email");
        println!("{}", email);
    }

    Ok(())
}
