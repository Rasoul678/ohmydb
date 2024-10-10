#![allow(dead_code)]

use colored::customcolors::CustomColor;
use colored::Colorize;
use fake::faker::internet::en::SafeEmail;
use fake::faker::lorem::en::{Sentence, Words};
use fake::faker::name::en::{FirstName, LastName};
use fake::Dummy;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Dummy, Eq, Hash, Default)]
pub enum Status {
    Pending,
    Completed,
    #[default]
    Archived,
}

impl Display for ToDo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n   {}: {},\n   {}: {},\n   {}: \"{}\",\n   {}: {},\n   {}: {},\n   {}\n {}",
            "{".bright_green().bold(),
            "id".bright_yellow().bold(),
            self.id.bright_cyan(),
            "status".bright_yellow().bold(),
            self.status.to_string().bright_cyan(),
            "text".bright_yellow().bold(),
            self.text.bright_cyan(),
            "point".bright_yellow().bold(),
            self.point.to_string().bright_cyan(),
            "user".bright_yellow().bold(),
            self.user.to_string().bright_cyan(),
            "...".bright_yellow().bold(),
            "}".bright_green().bold()
        )
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Pending => write!(f, "Status({})", "Pending".bright_purple().bold()),
            Status::Completed => write!(f, "Status({})", "Completed".bright_green().bold()),
            Status::Archived => write!(f, "Status({})", "Archived".yellow().bold()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Dummy, Eq, Hash)]
pub struct ToDo {
    pub id: String,
    #[dummy(faker = "Sentence(5..10)")]
    pub text: String,
    pub status: Status,
    pub user: User,
    pub date: Date,
    #[dummy(faker = "100..1000")]
    pub point: u64,
    #[dummy(faker = "Words(2..5)")]
    pub tags: Vec<String>,
}

impl Default for ToDo {
    fn default() -> Self {
        Self {
            id: "1".to_string(),
            text: "Learn Rust".to_string(),
            status: Status::Pending,
            user: User {
                name: Name {
                    first: "John".to_string(),
                    last: "Doe".to_string(),
                },
                email: "admin@gmail.com".to_string(),
            },
            date: Date {
                start: "2024-01-01".to_string(),
                end: "2025-01-01".to_string(),
            },
            tags: vec!["rust".to_string(), "programming".to_string()],
            point: 100,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Dummy, Eq, Hash, Default)]
pub struct User {
    pub name: Name,
    #[dummy(faker = "SafeEmail()")]
    pub email: String,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\n      {}: {},\n      {}: \"{}\",\n   }}",
            "name".yellow().bold(),
            self.name,
            "email".yellow().bold(),
            self.email,
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Dummy, Eq, Hash, Default)]
pub struct Name {
    #[dummy(faker = "FirstName()")]
    pub first: String,
    #[dummy(faker = "LastName()")]
    pub last: String,
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\n\t{}: \"{}\",\n\t{}: \"{}\",\n      }}",
            "first".yellow().bold(),
            self.first,
            "last".yellow().bold(),
            self.last,
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Dummy, Eq, Hash, Default)]
pub struct Date {
    pub start: String,
    pub end: String,
}

#[derive(Debug)]
pub enum Data {
    SingleTodo(ToDo),
    ListOfTodos(Vec<ToDo>),
}

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
    Create(String, ToDo, bool),
    Read(String),
    Update(String, ToDo),
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
            MethodName::Create(table, item, _) => println!(
                "{lead} {} {trail}\n\n {} \n",
                table.custom_color(gold).bold(),
                item,
                lead = "🌱 Creating a new record in".custom_color(green).bold(),
                trail = "table...".custom_color(green).bold()
            ),
            MethodName::Read(table) => println!(
                "{lead} {} {trail}\n",
                table.custom_color(gold).bold(),
                lead = "🔎 Querying".custom_color(teal).bold(),
                trail = "table...".custom_color(teal).bold()
            ),
            MethodName::Update(table, item) => println!(
                "{lead} {} {trail}\n\n {} \n",
                table.custom_color(gold).bold(),
                item,
                lead = "⛁ Updating a record in".custom_color(yellow).bold(),
                trail = "table...".custom_color(yellow).bold()
            ),
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
