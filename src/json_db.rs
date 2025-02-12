use crate::get_nested_value;
use crate::types::Comparator::{self, Between, Equals, GreaterThan, In, LessThan, NotEquals};
use crate::types::MethodName::{self, Create, Delete, Read, Update};
use crate::types::Runner::{self, Compare, Done, Method, Where};
use colored::*;
use serde::Serialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::ErrorKind::{AlreadyExists, InvalidData, NotFound};
use std::io::{Error, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone)]
pub struct JsonDB {
    tables: HashSet<String>,
    path: PathBuf,
    _file: Arc<File>,
    value: Arc<HashMap<String, HashSet<Value>>>,
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
    pub async fn new(db_name: &str) -> Result<Self> {
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
            serde_json::from_str(&content).map_err(|e| Error::new(InvalidData, e))?
        };

        let db = Self {
            tables: HashSet::new(),
            path: file_path,
            _file: Arc::new(file),
            value: Arc::new(value),
            runners: Arc::new(VecDeque::new()),
        };

        Ok(db)
    }

    pub fn get_db_path(&self) -> &str {
        self.path.as_os_str().to_str().unwrap_or_default()
    }

    pub async fn get_db_tables(&self) -> Vec<String> {
        let mut content = String::new();

        let file = OpenOptions::new().read(true).open(&self.path).await.ok();

        let tables = if file.is_some() {
            file.unwrap().read_to_string(&mut content).await.unwrap();

            let tables_hash: HashMap<String, HashSet<Value>> = serde_json::from_str(&content)
                .map_err(|e| Error::new(InvalidData, e))
                .unwrap_or_default();

            tables_hash.into_keys().collect::<Vec<String>>()
        } else {
            vec![]
        };

        tables
    }

    pub fn get_db_values(&self) -> Vec<(String, Vec<Value>)> {
        Arc::clone(&self.value)
            .iter()
            .map(|table| {
                let (t_name, t_records_hash) = table;
                let t_records_vec = t_records_hash
                    .iter()
                    .map(Clone::clone)
                    .collect::<Vec<Value>>();
                (t_name.clone(), t_records_vec)
            })
            .collect::<Vec<(String, Vec<Value>)>>()
    }

    /// Retrieves a mutable reference to the HashSet of `T` items for the specified table in the JSON database.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table to retrieve the mutable reference for.
    ///
    /// # Returns
    ///
    /// A `Result` containing a mutable reference to the `HashSet<T>` for the specified table if it exists,
    /// or an `io::Error` if the table is not found.
    fn get_table_mut(&mut self, table_name: &str) -> Result<&mut HashSet<Value>> {
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
                Error::new(NotFound, format!("Table '{}' not found", table_name))
            })?;

        Ok(table)
    }

    /// Retrieves a vector of `T` items from the specified table in the JSON database.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table to retrieve the items from.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<T>` if the table is found, or an `io::Error` if the table is not found.
    pub fn get_table_vec(&mut self, table_name: &str) -> Result<Vec<Value>> {
        let hash_table = (*self.value)
            .clone()
            .get(table_name)
            .map(Clone::clone)
            .ok_or_else(|| Error::new(NotFound, format!("Table '{}' not found", table_name)))?;

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
    pub async fn add_table(&mut self, table_name: &str) -> Result<()> {
        let tables_hash = Arc::make_mut(&mut self.value);

        let table_already_exists = tables_hash.contains_key(table_name);

        if !table_already_exists {
            tables_hash.insert(table_name.to_string(), HashSet::new());
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
    pub async fn save(&self) -> Result<()> {
        let json =
            serde_json::to_string_pretty(&*self.value).map_err(|e| Error::new(InvalidData, e))?;

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
    /// * `item` - The `T` item to insert.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `JsonDb` instance, allowing for method chaining.
    pub fn insert<T>(&mut self, table: &str, item: &T) -> &mut Self
    where
        T: Serialize,
    {
        let value = serde_json::to_value(item).unwrap();
        Arc::make_mut(&mut self.runners).push_back(Method(Create(table.to_string(), value, false)));
        self
    }

    /// Inserts a new record into the JSON database table,
    /// or creates a table first if it does not already exists.
    ///
    /// # Arguments
    ///
    /// * `table` - The name of the table to insert the record into.
    /// * `item` - The `T` item to insert or update.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `JsonDb` instance, allowing for method chaining.
    pub fn insert_or<T>(&mut self, table: &str, item: &T) -> &mut Self
    where
        T: Serialize,
    {
        let value = serde_json::to_value(item).unwrap();
        Arc::make_mut(&mut self.runners).push_back(Method(Create(table.to_string(), value, true)));
        self
    }

    /// Adds a `Runner::Method(MethodName::Read)` to the end of the runners queue, indicating that the current operation is a read operation.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn find(&mut self, table: &str) -> &mut Self {
        Arc::make_mut(&mut self.runners).push_back(Method(Read(table.to_string())));

        self
    }

    /// Adds a `Runner::Method(MethodName::Update)` to the end of the runners queue, indicating that the current operation is an update operation.
    /// The returned `Self` instance contains the updated runners queue.
    ///
    /// # Returns
    ///
    /// A new `Self` instance with the updated runners queue.
    pub fn update<T>(&mut self, table: &str, item: &T) -> &mut Self
    where
        T: Serialize,
    {
        let value = serde_json::to_value(item).unwrap();
        Arc::make_mut(&mut self.runners).push_back(Method(Update(table.to_string(), value)));

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
        Arc::make_mut(&mut self.runners).push_back(Method(Delete(table.to_string())));

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
        Arc::make_mut(&mut self.runners).push_back(Where(field.to_string()));

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
        Arc::make_mut(&mut self.runners).push_back(Compare(Equals(value.to_string())));

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
        Arc::make_mut(&mut self.runners).push_back(Compare(NotEquals(value.to_string())));

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
        Arc::make_mut(&mut self.runners).push_back(Compare(In(values)));

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
        Arc::make_mut(&mut self.runners).push_back(Compare(LessThan(value)));

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
        Arc::make_mut(&mut self.runners).push_back(Compare(GreaterThan(value)));

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
        Arc::make_mut(&mut self.runners).push_back(Compare(Between((start, end))));

        self
    }

    /// Runs the database operations specified in the runners queue.
    ///
    /// This method processes the runners queue, performing various database operations such as creating, reading, updating, and deleting records.
    /// The method returns the resulting list of `T` items after applying the specified operations.
    ///
    /// # Errors
    ///
    /// This method may return an `std::io::Error` if there is an error saving the database state after the operations are completed.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec` of `T` items representing the final state of the database after the operations have been performed.
    pub async fn run(&mut self) -> Result<Vec<Value>> {
        let mut result = Vec::new();
        let mut key_chain = String::new();
        let mut method: Option<MethodName> = None;

        Arc::make_mut(&mut self.runners).push_back(Done);

        while let Some(runner) = Arc::make_mut(&mut self.runners).pop_front() {
            match runner {
                Method(name) => match name {
                    Create(table, new_item, or) => {
                        result = self.get_table_vec(&table).unwrap_or_default();
                        method = Some(Create(table, new_item.clone(), or));
                    }
                    Read(table) => {
                        result = self.get_table_vec(&table).unwrap_or_default();
                        method = Some(Read(table));
                    }
                    Delete(table) => {
                        result = self.get_table_vec(&table).unwrap_or_default();
                        method = Some(Delete(table));
                    }
                    Update(table, new_item) => {
                        result = self.get_table_vec(&table).unwrap_or_default();
                        method = Some(Update(table, new_item));
                    }
                },
                Where(f) => {
                    key_chain = f;
                }
                Compare(ref comparator) => {
                    result = result
                        .into_iter()
                        .filter(|t| {
                            let value = get_nested_value(t, &key_chain).unwrap();
                            self.filter_with_conmpare(value, comparator)
                        })
                        .collect();
                }
                Done => {
                    match method {
                        Some(Read(table)) => {
                            Read(table).notify();
                        }
                        Some(Create(table, ref new_item, or)) => {
                            self.insert_into_table(table.as_str(), &new_item, or)?;
                            Create(table, new_item.clone(), or).notify();
                        }
                        Some(Update(table, new_item)) => {
                            let new_item_id: Value =
                                get_nested_value(new_item.clone(), "id").unwrap();
                            let search_result = result
                                .iter()
                                .find(|t| {
                                    let current_item_id: Value = get_nested_value(t, "id").unwrap();
                                    current_item_id.as_str().unwrap()
                                        == new_item_id.as_str().unwrap()
                                })
                                .ok_or(Error::new(
                                    NotFound,
                                    format!(
                                        "Schade! Record with id \"{}\" not found in table {}",
                                        new_item_id.as_str().unwrap(),
                                        table.bright_cyan().bold()
                                    ),
                                ));

                            match search_result {
                                Ok(search_value) => {
                                    let table_hash = self.get_table_mut(&table)?;
                                    let search_value_id: Value =
                                        get_nested_value(search_value, "id").unwrap();

                                    table_hash.retain(|t| {
                                        let current_id: Value = get_nested_value(t, "id").unwrap();
                                        current_id.as_str().unwrap()
                                            != search_value_id.as_str().unwrap()
                                    });

                                    table_hash.insert(new_item.clone());

                                    result.clear();
                                    result.push(new_item.clone());

                                    Update(table, new_item.to_owned()).notify();
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
                        Some(Delete(table)) => {
                            let table_hash = self.get_table_mut(&table)?;

                            for r in result.iter() {
                                table_hash.retain(|t| {
                                    let t_id: Value = get_nested_value(t, "id").unwrap();
                                    let r_id: Value = get_nested_value(r, "id").unwrap();
                                    t_id.as_str().unwrap() != r_id.as_str().unwrap()
                                });
                            }

                            Delete(table).notify();
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
            Equals(v) => value.as_str() == Some(v.as_str()),
            NotEquals(v) => value.as_str() != Some(v.as_str()),
            LessThan(v) => value.as_u64().map_or(false, |x| x < *v),
            GreaterThan(v) => value.as_u64().map_or(false, |x| x > *v),
            In(vs) => value
                .as_str()
                .map_or(false, |x| vs.contains(&x.to_string())),
            Between((start, end)) => value.as_u64().map_or(false, |x| x >= *start && x <= *end),
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
    /// * `Result<&'a T, io::Error>` - A result containing either a reference to the inserted item or an error if the item already exists.
    fn insert_into_table<'a>(
        &mut self,
        table_name: &str,
        new_item: &'a Value,
        or: bool,
    ) -> Result<&'a Value> {
        let new_item_id: Value = get_nested_value(new_item, "id").unwrap();

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
                new_item_id.as_str().unwrap().bright_red().bold(),
                "\" already exists in table".bright_red().bold(),
                table_name.to_string().bright_cyan().bold(),
                "✔".bright_green().bold().blink(),
                "Try to add new record".bright_green().bold()
            );
            return Err(Error::new(AlreadyExists, "Record already exists"));
        }

        // Check for double entries with same id
        let search_table = table.iter().find(|t| {
            let current_id: Value = get_nested_value(t, "id").unwrap();

            current_id.as_str().unwrap() == new_item_id.as_str().unwrap()
        });

        match search_table {
            Some(t) => {
                let t_id: Value = get_nested_value(t, "id").unwrap();

                return Err(Error::new(
                    AlreadyExists,
                    format!(
                        "Record with id \"{}\" already exists",
                        t_id.as_str().unwrap()
                    ),
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
