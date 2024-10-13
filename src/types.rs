#![allow(dead_code)]

use colored::customcolors::CustomColor;
use colored::Colorize;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::hash::Hash;

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
pub enum MethodName<T>
where
    T: Serialize + DeserializeOwned + Clone + Debug + PartialEq + Eq + Hash,
{
    Create(String, T, bool),
    Read(String),
    Update(String, T),
    Delete(String),
}

impl<T> MethodName<T>
where
    T: Serialize + DeserializeOwned + Clone + Debug + PartialEq + Eq + Hash,
{
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
                "{lead} {} {trail}\n\n {:#?} \n",
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
                "{lead} {} {trail}\n\n {:#?} \n",
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
pub enum Runner<T>
where
    T: Serialize + DeserializeOwned + Clone + Debug + PartialEq + Eq + Hash,
{
    Done,
    Method(MethodName<T>),
    Compare(Comparator),
    Where(String),
}
