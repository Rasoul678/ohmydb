#![allow(dead_code)]

use crate::utils::display_object;
use colored::customcolors::CustomColor;
use colored::Colorize;
use serde_json::Value;
use std::fmt::Debug;

#[derive(Clone, PartialEq, Debug)]
pub enum Comparator {
    Equals(String),
    NotEquals(String),
    LessThan(u64),
    GreaterThan(u64),
    In(Vec<String>),
    Between((u64, u64)),
}

#[derive(Clone, PartialEq, Debug)]
pub enum MethodName {
    Create(String, Value, bool),
    Read(String),
    Update(String, Value),
    Delete(String),
}

impl MethodName {
    /// Prints a message to the console based on the variant of the `MethodName` enum.
    ///
    /// This method is used to provide visual feedback to the user when performing CRUD operations on a database table.
    /// The message includes the table name, the item being operated on, and a colored prefix indicating the type of operation.
    ///
    /// # Examples
    ///
    /// let method_name = MethodName::Create("users_table".to_string(), todo, false);
    /// method_name.notify();
    ///
    /// This will print a message like:
    ///
    /// 🌱 Creating a new record in USERS_TABLE table...
    ///
    /// { "first": "John", "last": "Doe" }
    pub fn notify(&self) {
        let teal = CustomColor::new(0, 201, 217);
        let gold = CustomColor::new(251, 190, 13);
        let green = CustomColor::new(8, 171, 112);
        let yellow = CustomColor::new(242, 140, 54);
        let red = CustomColor::new(217, 33, 33);

        match self {
            MethodName::Create(table, item, _) => {
                if let Value::Object(obj) = item {
                    println!(
                        "{lead} {} {trail}\n\n {} \n",
                        table.custom_color(gold).bold(),
                        display_object(obj, 1),
                        lead = "🌱 Creating a new record in".custom_color(green).bold(),
                        trail = "table...".custom_color(green).bold()
                    )
                } else {
                    println!("Not a JSON object");
                }
            }
            MethodName::Read(table) => println!(
                "{lead} {} {trail}\n",
                table.custom_color(gold).bold(),
                lead = "🔎 Querying".custom_color(teal).bold(),
                trail = "table...".custom_color(teal).bold()
            ),
            MethodName::Update(table, item) => {
                if let Value::Object(obj) = item {
                    println!(
                        "{lead} {} {trail}\n\n {} \n",
                        table.custom_color(gold).bold(),
                        display_object(obj, 1),
                        lead = "⛁ Updating a record in".custom_color(yellow).bold(),
                        trail = "table...".custom_color(yellow).bold()
                    )
                } else {
                    println!("Not a JSON object");
                }
            }
            MethodName::Delete(table) => println!(
                "{lead} {} {trail}\n",
                table.custom_color(gold).bold(),
                lead = "✗ Deleting records from".custom_color(red).bold(),
                trail = "table...".custom_color(red).bold()
            ),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Runner {
    Done,
    Method(MethodName),
    Compare(Comparator),
    Where(String),
}

struct MyType {
    name: String,
    age: u32,
}

impl From<Value> for MyType {
    fn from(value: Value) -> Self {
        match value {
            Value::Object(map) => {
                let name = map.get("name").unwrap().as_str().unwrap().to_string();
                let age = map.get("age").unwrap().as_u64().unwrap() as u32;
                MyType { name, age }
            }
            _ => panic!("Invalid value"),
        }
    }
}
