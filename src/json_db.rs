#![allow(dead_code)]

use crate::get_nested_value;
use crate::types::{Comparator, MethodName, Runner, ToDo};
use colored::*;
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{self, ErrorKind};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone)]
pub struct JsonDB {
    tables: HashSet<String>,
    path: PathBuf,
    file: Arc<File>,
    value: Arc<HashMap<String, HashSet<ToDo>>>,
    runners: Arc<VecDeque<Runner>>,
}

impl JsonDB {
    /// Creates a new instance of the `JsonDB` struct, initializing it with a new JSON database file.
    ///
    /// This function reads the contents of the `db.json` file in the current directory,
    /// or creates a new file if it doesn't exist. The file contents are deserialized into a `HashMap` and stored in the `JsonDB` struct.
    /// The `JsonDB` struct also initializes an empty `HashSet` for table names, an `Arc`-wrapped `File` instance, and an empty `VecDeque` for runners.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `JsonDB` instance if the operation is successful,
    /// or an `io::Error` if there is a problem reading or creating the file.
    pub async fn new(db_name: &str) -> Result<JsonDB, io::Error> {
        let db_path;

        if db_name.is_empty() {
            db_path = format!("ohmydb.json")
        } else {
            db_path = format!("{}.json", db_name.to_lowercase().trim())
        }

        let dir_path = std::env::current_dir()?;
        let file_path = dir_path.join(db_path);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)
            .await?;

        let mut content = String::new();

        file.try_clone().await?.read_to_string(&mut content).await?;
        // let mut value = HashMap::new();

        let value = if content.is_empty() {
            HashMap::new()
        } else {
            serde_json::from_str(&content).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?
        };

        let db = JsonDB {
            tables: HashSet::new(),
            path: file_path,
            file: Arc::new(file),
            value: Arc::new(value),
            runners: Arc::new(VecDeque::new()),
        };

        Ok(db)
    }

    pub fn get_db_path(&self) -> &str {
        self.path.as_os_str().to_str().unwrap_or_default()
    }

    pub fn get_db_tables(&self) -> Vec<String> {
        self.tables
            .iter()
            .map(Clone::clone)
            .collect::<Vec<String>>()
    }

    pub fn get_db_values(&self) -> Vec<(String, Vec<ToDo>)> {
        Arc::clone(&self.value)
            .iter()
            .map(|table| {
                let (t_name, t_records_hash) = table;
                let t_records_vec = t_records_hash
                    .iter()
                    .map(Clone::clone)
                    .collect::<Vec<ToDo>>();
                (t_name.clone(), t_records_vec)
            })
            .collect::<Vec<(String, Vec<ToDo>)>>()
    }

    /// Retrieves a mutable reference to the HashSet of `ToDo` items for the specified table in the JSON database.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table to retrieve the mutable reference for.
    ///
    /// # Returns
    ///
    /// A `Result` containing a mutable reference to the `HashSet<ToDo>` for the specified table if it exists,
    /// or an `io::Error` if the table is not found.
    fn get_table_mut(&mut self, table_name: &str) -> Result<&mut HashSet<ToDo>, io::Error> {
        let table = Arc::make_mut(&mut self.value)
            .get_mut(table_name)
            .ok_or_else(|| {
                println!(
                    "{} {} \"{}\" {}\n\t\t{} {}\n",
                    "(get_table_mut)".bright_cyan().bold(),
                    "✗ Retrieving".bright_red().bold(),
                    table_name.to_string().bright_red().bold(),
                    "table failed!".bright_red().bold(),
                    "✔".bright_green().bold().blink(),
                    "Try to add a table first!".bright_green().bold()
                );
                io::Error::new(
                    ErrorKind::NotFound,
                    format!("Table '{}' not found", table_name),
                )
            })?;

        Ok(table)
    }

    /// Retrieves a vector of `ToDo` items from the specified table in the JSON database.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table to retrieve the items from.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<ToDo>` if the table is found, or an `io::Error` if the table is not found.
    fn get_table_vec(&mut self, table_name: &str) -> Result<Vec<ToDo>, io::Error> {
        let hash_table = (*self.value)
            .clone()
            .get(table_name)
            .map(Clone::clone)
            .ok_or_else(|| {
                io::Error::new(
                    ErrorKind::NotFound,
                    format!("Table '{}' not found", table_name),
                )
            })?;

        let table = Vec::from_iter(hash_table);

        Ok(table)
    }

    /// Adds a new table to the JSON database.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table to add.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the table was successfully added. If the table already exists, this function will return `Ok(())`.
    pub async fn add_table(&mut self, table_name: &str) -> Result<(), io::Error> {
        let value = Arc::make_mut(&mut self.value);

        if !value.contains_key(table_name) {
            value.insert(table_name.to_string(), HashSet::new());
            self.tables.insert(table_name.to_string());
        }

        self.save().await?;

        Ok(())
    }

    /// Saves the current state of the `JsonDb` instance to the file specified by the `path` field.
    ///
    /// # Errors
    ///
    /// This function will return an error if there is a problem writing the JSON data to the file.
    pub async fn save(&self) -> Result<(), io::Error> {
        let json = serde_json::to_string_pretty(&*self.value)
            .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .await?;

        file.write_all(json.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }

    /// Inserts a new record into the JSON database table.
    ///
    /// # Arguments
    ///
    /// * `table` - The name of the table to insert the record into.
    /// * `item` - The `ToDo` item to insert.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `JsonDb` instance, allowing for method chaining.
    pub fn insert(&mut self, table: &str, item: ToDo) -> &mut Self {
        Arc::make_mut(&mut self.runners).push_back(Runner::Method(MethodName::Create(
            table.to_string(),
            item,
            false,
        )));
        self
    }

    /// Inserts a new record into the JSON database table,
    /// or creates a table first if it does not already exists.
    ///
    /// # Arguments
    ///
    /// * `table` - The name of the table to insert the record into.
    /// * `item` - The `ToDo` item to insert or update.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `JsonDb` instance, allowing for method chaining.
    pub fn insert_or(&mut self, table: &str, item: ToDo) -> &mut Self {
        Arc::make_mut(&mut self.runners).push_back(Runner::Method(MethodName::Create(
            table.to_string(),
            item,
            true,
        )));
        self
    }

    /// Adds a `Runner::Method(MethodName::Read)` to the end of the runners queue, indicating that the current operation is a read operation.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn find(&mut self, table: &str) -> &mut Self {
        Arc::make_mut(&mut self.runners)
            .push_back(Runner::Method(MethodName::Read(table.to_string())));

        self
    }

    /// Adds a `Runner::Method(MethodName::Update)` to the end of the runners queue, indicating that the current operation is an update operation.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn update(&mut self, table: &str, data: ToDo) -> &mut Self {
        Arc::make_mut(&mut self.runners)
            .push_back(Runner::Method(MethodName::Update(table.to_string(), data)));

        self
    }

    /// Adds a `Runner::Method(MethodName::Delete(c))` to the end of the runners queue,
    /// indicating that the current operation is a delete operation.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `key` - The character to use for the delete operation.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn delete(&mut self, table: &str) -> &mut Self {
        Arc::make_mut(&mut self.runners)
            .push_back(Runner::Method(MethodName::Delete(table.to_string())));

        self
    }

    /// Adds a `Runner::Where(field.to_string())` to the end of the runners queue, filtering the data based on the provided field.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `field` - The field to filter the data by.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn where_(&mut self, field: &str) -> &mut Self {
        Arc::make_mut(&mut self.runners).push_back(Runner::Where(field.to_string()));

        self
    }

    /// Adds a `Runner::Compare(Comparator::Equals(value.to_string()))` to the end of the runners queue, filtering the data based on the provided value.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to filter the data by.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn equals(&mut self, value: &str) -> &mut Self {
        Arc::make_mut(&mut self.runners)
            .push_back(Runner::Compare(Comparator::Equals(value.to_string())));

        self
    }

    /// Adds a `Runner::Compare(Comparator::NotEquals(value.to_string()))` to the end of the runners queue, filtering the data based on the provided value.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to filter the data by.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn not_equals(&mut self, value: &str) -> &mut Self {
        Arc::make_mut(&mut self.runners)
            .push_back(Runner::Compare(Comparator::NotEquals(value.to_string())));

        self
    }

    /// Adds a `Runner::Compare(Comparator::In(value.to_vec()))` to the end of the runners queue, filtering the data based on the provided values.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `value` - The values to filter the data by.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn in_(&mut self, values: Vec<String>) -> &mut Self {
        Arc::make_mut(&mut self.runners).push_back(Runner::Compare(Comparator::In(values)));

        self
    }

    /// Adds a `Runner::Compare(Comparator::LessThan(value))` to the end of the runners queue, filtering the data based on the provided value.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to filter the data by.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn less_than(&mut self, value: u64) -> &mut Self {
        Arc::make_mut(&mut self.runners).push_back(Runner::Compare(Comparator::LessThan(value)));

        self
    }

    /// Adds a `Runner::Compare(Comparator::GreaterThan(value))` to the end of the runners queue, filtering the data based on the provided value.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to filter the data by.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn greater_than(&mut self, value: u64) -> &mut Self {
        Arc::make_mut(&mut self.runners).push_back(Runner::Compare(Comparator::GreaterThan(value)));

        self
    }

    /// Adds a `Runner::Compare(Comparator::Between((start, end)))` to the end of the runners queue, filtering the data based on the provided start and end values.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Arguments
    ///
    /// * `start` - The start value to filter the data by.
    /// * `end` - The end value to filter the data by.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn between(&mut self, start: u64, end: u64) -> &mut Self {
        Arc::make_mut(&mut self.runners)
            .push_back(Runner::Compare(Comparator::Between((start, end))));

        self
    }

    /// Runs the database operations specified in the runners queue.
    ///
    /// This method processes the runners queue, performing various database operations such as creating, reading, updating, and deleting records.
    /// The method returns the resulting list of `ToDo` items after applying the specified operations.
    ///
    /// # Errors
    ///
    /// This method may return an `std::io::Error` if there is an error saving the database state after the operations are completed.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec` of `ToDo` items representing the final state of the database after the operations have been performed.
    pub async fn run(&mut self) -> Result<Vec<ToDo>, std::io::Error> {
        let mut result = Vec::new();
        let mut key_chain = String::new();
        let mut method: Option<MethodName> = None;

        Arc::make_mut(&mut self.runners).push_back(Runner::Done);

        while let Some(runner) = Arc::make_mut(&mut self.runners).pop_front() {
            match runner {
                Runner::Method(name) => match name {
                    MethodName::Create(table, new_item, or) => {
                        result = self.get_table_vec(&table).unwrap_or_default();
                        method = Some(MethodName::Create(table, new_item.clone(), or));
                    }
                    MethodName::Read(table) => {
                        result = self.get_table_vec(&table).unwrap_or_default();
                        method = Some(MethodName::Read(table));
                    }
                    MethodName::Delete(table) => {
                        result = self.get_table_vec(&table).unwrap_or_default();
                        method = Some(MethodName::Delete(table));
                    }
                    MethodName::Update(table, new_item) => {
                        result = self.get_table_vec(&table).unwrap_or_default();
                        method = Some(MethodName::Update(table, new_item));
                    }
                },
                Runner::Where(f) => {
                    key_chain = f;
                }
                Runner::Compare(ref comparator) => {
                    result = result
                        .into_iter()
                        .filter(|todo| {
                            let value: Value = get_nested_value(todo, &key_chain).unwrap();
                            self.filter_with_conmpare(value, comparator)
                        })
                        .collect();
                }
                Runner::Done => {
                    match method {
                        Some(MethodName::Read(table)) => {
                            MethodName::Read(table).notify();
                        }
                        Some(MethodName::Create(table, ref new_item, or)) => {
                            self.insert_into_table(table.as_str(), &new_item, or)?;
                            MethodName::Create(table, new_item.clone(), or).notify();
                        }
                        Some(MethodName::Update(table, new_item)) => {
                            let search_result =
                                result
                                    .iter()
                                    .find(|t| t.id == new_item.id)
                                    .ok_or(io::Error::new(
                                        ErrorKind::NotFound,
                                        format!(
                                            "Schade! Record with id \"{}\" not found in table {}",
                                            new_item.id,
                                            table.bright_cyan().bold()
                                        ),
                                    ));

                            match search_result {
                                Ok(old_todo) => {
                                    let table_hash = self.get_table_mut(&table)?;

                                    table_hash.retain(|t| t.id != old_todo.id);
                                    table_hash.insert(new_item.clone());

                                    result.clear();
                                    result.push(new_item.clone());

                                    MethodName::Update(table, new_item.to_owned()).notify();
                                }

                                Err(err) => {
                                    println!(
                                        "{}  {} {}\n\t\t{} {}\n",
                                        "(update_table)".bright_cyan().bold(),
                                        "✗".bright_red().bold(),
                                        err.to_string().bright_red().bold(),
                                        "✔".bright_green().bold().blink(),
                                        "Consider adding new record".bright_green().bold()
                                    );
                                    return Err(err);
                                }
                            };
                        }
                        Some(MethodName::Delete(table)) => {
                            let table_hash = self.get_table_mut(&table)?;

                            for t in result.iter() {
                                table_hash.retain(|todo| todo.id != t.id);
                            }

                            MethodName::Delete(table).notify();
                        }
                        _ => {}
                    }

                    self.save().await?;

                    break;
                }
            }
        }

        Ok(result)
    }

    /// Filters a `Value` based on the provided `Comparator`.
    ///
    /// This function takes a `Value` and a `Comparator` and returns a boolean indicating whether the `Value` matches the comparison criteria.
    ///
    /// # Examples
    ///
    /// use serde_json::Value;
    /// use json_db::Comparator;
    ///
    /// let json_db = JsonDB::new();
    /// let value = Value::from(42u64);
    /// let comparator = Comparator::GreaterThan(30);
    /// assert!(json_db.filter_with_conmpare(value, &comparator));
    ///
    fn filter_with_conmpare(&self, value: Value, comparator: &Comparator) -> bool {
        match comparator {
            Comparator::Equals(v) => value.as_str() == Some(v.as_str()),
            Comparator::NotEquals(v) => value.as_str() != Some(v.as_str()),
            Comparator::LessThan(v) => value.as_u64().map_or(false, |x| x < *v),
            Comparator::GreaterThan(v) => value.as_u64().map_or(false, |x| x > *v),
            Comparator::In(vs) => value
                .as_str()
                .map_or(false, |x| vs.contains(&x.to_string())),
            Comparator::Between((start, end)) => {
                value.as_u64().map_or(false, |x| x >= *start && x <= *end)
            }
        }
    }

    /// Inserts a new item into a table in the JSON database.
    ///
    /// This function takes a table name, a new item to insert,
    /// and a boolean flag indicating whether to create the table if it doesn't exist.
    /// If the new item already exists in the table, either by exact properties or by ID, an error is returned.
    /// Otherwise, the new item is inserted into the table and a reference to the inserted item is returned.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table to insert the new item into.
    /// * `new_item` - The new item to insert into the table.
    /// * `or` - A boolean flag indicating whether to create the table if it doesn't exist.
    ///
    /// # Returns
    ///
    /// * `Result<&'a ToDo, io::Error>` - A result containing either a reference to the inserted item or an error if the item already exists.
    fn insert_into_table<'a>(
        &mut self,
        table_name: &str,
        new_item: &'a ToDo,
        or: bool,
    ) -> Result<&'a ToDo, io::Error> {
        let table = if or {
            let db_hash = Arc::make_mut(&mut self.value);

            match db_hash.get_mut(table_name) {
                Some(t) => t,
                None => {
                    self.tables.insert(table_name.to_string());
                    db_hash.insert(table_name.to_string(), HashSet::new());
                    db_hash.get_mut(table_name).unwrap()
                }
            }
        } else {
            self.get_table_mut(table_name)?
        };

        // Check if the new item already exists in the set for exact same properties
        if table.contains(new_item) {
            println!(
                "{} {}{}{} {}\n\t\t    {} {}\n",
                "(insert_into_table)".bright_cyan().bold(),
                "✗ Schade! Record with id \"".bright_red().bold(),
                new_item.id.bright_red().bold(),
                "\" already exists in table".bright_red().bold(),
                table_name.to_string().bright_cyan().bold(),
                "✔".bright_green().bold().blink(),
                "Try to add new record".bright_green().bold()
            );
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Record already exists",
            ));
        }

        // Check for double entries with same id
        let search_table = table.iter().find(|t| t.id == new_item.id);

        match search_table {
            Some(t) => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("Record with id \"{}\" already exists", t.id),
                ));
            }
            None => {
                // Insert the new item
                table.insert(new_item.clone());
            }
        }

        Ok(new_item)
    }
}
