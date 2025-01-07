use ohmydb::{define_struct_from, JsonDB};

define_struct_from!(Todo, {id: String, assignee: String, title: String, is_completed: bool, created_at: String, updated_at: String});

#[tokio::main]
async fn main() {
    println!("{}", "=".repeat(80));
    println!("JsonDB!");
    println!("{}", "=".repeat(80));
    let mut db: JsonDB<Todo> = JsonDB::new("test").await.unwrap();
    db.add_table("users").await.unwrap();
    db.add_table("files").await.unwrap();

    let todo = Todo {
        id: "1".to_string(),
        title: "Buy groceries".to_string(),
        assignee: "John Doe".to_string(),
        is_completed: false,
        created_at: "2023-06-01".to_string(),
        updated_at: "2023-06-01".to_string(),
    };

    db.insert("users", &todo).run().await.unwrap();
    db.insert("files", &todo).run().await.unwrap();

    let tables = db.get_db_tables().await;

    println!("Tables: {:#?}", tables);
    println!("{}", "=".repeat(80));
}
