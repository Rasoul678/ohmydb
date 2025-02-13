# Ohmydb - A Rust json Database

[![Crates.io](https://img.shields.io/crates/v/crate-name)](https://crates.io/crates/ohmydb)
[![Documentation](https://docs.rs/crate-name/badge.svg)](https://docs.rs/ohmydb/latest/ohmydb/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Installation](#installation)
- [Example](#example)
- [API Documentation](#api-documentation)
- [Contributing](#contributing)
- [License](#license)

## Introduction

Ohmydb is a Rust library for working with JSON data. It provides a simple and efficient way to store, retrieve, and manipulate JSON data.

## Features

- Feature 1: Lightweight and efficient JSON storage
- Feature 2: Blazingly fast JSON operations
- Feature 3: Easy-to-use API for JSON manipulation

## Installation

Add this to your `Cargo.toml`:

[dependencies]
ohmydb = "2.1.1"

# Example

```rust
use ohmydb::{define_struct_from, JsonDB};

define_struct_from!(
    User {
        id: String,
        name: String,
        occupation: String,
        created_at: String,
        updated_at: String
    },
    Todo {
        id: String,
        assignee: String,
        title: String,
        is_completed: bool,
        created_at: String,
        updated_at: String,
        array: Vec<String>,
        wife: User
    }
);

#[tokio::main]
async fn main() {
    println!("{}", "=".repeat(80));
    println!("JsonDB!");
    println!("{}", "=".repeat(80));

    // ! Create a new instance of the `JsonDB` struct
    let mut db = JsonDB::new("test").await.unwrap();

    // ! Add tables to the database
    db.add_table("todos").await.unwrap();
    db.add_table("users").await.unwrap();

    let user = User {
        id: "1".to_string(),
        name: "Jane Doe".to_string(),
        occupation: "Software Engineer".to_string(),
        created_at: "2025-01-07".to_string(),
        updated_at: "2025-01-07".to_string(),
    };

    let todo = Todo {
        id: "1".to_string(),
        title: "Buy groceries".to_string(),
        assignee: "John Doe".to_string(),
        is_completed: false,
        created_at: "2025-01-07".to_string(),
        updated_at: "2025-01-07".to_string(),
        wife: user.clone(),
        array: vec!["John".to_string(), "Doe".to_string()],
    };

    // ! Insert data into the tables
    db.insert("todos", &todo).run().await.ok();
    db.insert("users", &user).run().await.ok();

    let updated_user = User {
        occupation: "Frontend Developer".to_string(),
        ..user
    };

    // ! Update data in the specified table
    db.update("users", &updated_user).run().await.ok();

    // ! Find data in the specified table
    let my_todo = db
        .find("todos")
        .where_("wife.name")
        .equals("Jane Doe")
        .run()
        .await
        .ok()
        .unwrap();

    println!("My Todo: {:#?}", my_todo);

    // ! Get the database tables
    let tables = db.get_db_tables().await;
    println!("Tables: {:#?}", tables);

    // ! Get the database path
    let path = db.get_db_path();
    println!("Path: {}", path);

    // ! Delete some datas from the specified table
    db.delete("users")
        .where_("name")
        .not_equals("Jane Doe")
        .run()
        .await
        .ok();

    // ! Delete all datas from the specified table
    db.delete("todos").run().await.ok();
}
```

## API Documentation

For detailed API documentation, please refer to the [API Documentation](https://docs.rs/ohmydb/latest/ohmydb/).

## Contributing

Contributions are welcome! If you find any issues or have suggestions for improvements, please open an issue or submit a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more information.
