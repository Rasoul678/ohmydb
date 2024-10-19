use ohmydb::JsonDB;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
struct Todo {
    id: u32,
    title: String,
    completed: bool,
    created_at: String,
    updated_at: String,
}

#[tokio::main]
async fn main() {
    println!("{}", "=".repeat(80));
    println!("JsonDB!");
    println!("{}", "=".repeat(80));
    let mut db: JsonDB<Todo> = JsonDB::new("test").await.unwrap();
    db.add_table("users").await.unwrap();
    db.add_table("files").await.unwrap();

    let tables = db.get_db_tables().await;

    println!("Tables: {:#?}", tables);
    println!("{}", "=".repeat(80));
}
