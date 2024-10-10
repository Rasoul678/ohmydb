use fake::{Fake, Faker};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_value::Value;
use std::collections::VecDeque;
use std::io::{Error, ErrorKind, Result};

/// Generates a vector of `T` instances with random data using the `fake` crate.
///
/// This function is useful for generating fake data for testing or prototyping purposes.
///
/// # Arguments
///
/// * `count` - The number of `T` instances to generate.
///
/// # Returns
///
/// A vector of `T` instances with randomly generated data.
///
/// # Example
///
///
/// use src::utils::fake_it;
/// use fake::Dummy;
///
/// #[derive(Dummy)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let people: Vec<Person> = fake_it(10);
///
pub fn fake_it<T>(count: u32) -> Vec<T>
where
    T: fake::Dummy<fake::Faker>,
{
    let mut list: Vec<T> = vec![];

    for _ in 0..count {
        let item = Faker.fake::<T>();

        list.push(item);
    }

    list
}

/// Retrieves the value of a field by name from a serializable data structure.
///
/// This function takes a serializable data structure `data` and a field name `field`,
/// and attempts to retrieve the value of the specified field. If the field is found,
/// it is deserialized into the desired type `R` and returned. If the field is not
/// found or the deserialization fails, an error is returned.
///
/// # Arguments
///
/// * `data` - The serializable data structure to retrieve the field from.
/// * `field` - The name of the field to retrieve.
///
/// # Returns
///
/// A `Result` containing the deserialized value of the field, or an error if the
/// field is not found or the deserialization fails.
pub fn get_field_by_name<T, R>(data: T, field: &str) -> Result<R>
where
    T: Serialize,
    R: DeserializeOwned,
{
    let mut map = match serde_value::to_value(data) {
        Ok(Value::Map(map)) => map,
        _ => {
            return Err(Error::new(ErrorKind::InvalidInput, "expected a struct"));
        }
    };

    let key = Value::String(field.to_owned());
    let value = match map.remove(&key) {
        Some(value) => value,
        None => return Err(Error::new(ErrorKind::NotFound, "no such field")),
    };

    match R::deserialize(value) {
        Ok(r) => Ok(r),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e.to_string())),
    }
}

/// Retrieves the value of a nested field in a serializable data structure.
///
/// This function takes a serializable data structure `data` and a dot-separated
/// `key_chain` that specifies the path to a nested field. It attempts to retrieve
/// the value of the specified field. If the field is found, it is returned as a
/// `Value`. If any part of the key chain is not found, `None` is returned.
///
/// # Arguments
///
/// * `data` - The serializable data structure to retrieve the field from.
/// * `key_chain` - A dot-separated string that specifies the path to the nested field.
///
/// # Returns
///
/// An `Option<Value>` containing the value of the specified nested field, or `None`
/// if any part of the key chain is not found.
pub fn get_key_chain_value<T>(data: T, key_chain: &str) -> Option<Value>
where
    T: Serialize,
{
    let mut parts = key_chain.split('.').collect::<Vec<&str>>();
    let key = parts.remove(0);
    let value: Value = get_field_by_name(data, key).unwrap();

    if parts.len() > 0 {
        let new_key_chain = parts.join(".");
        return get_key_chain_value(value, &new_key_chain);
    }

    Some(value)
}

/// Retrieves the value of a nested field in a serializable data structure.
///
/// This function takes a serializable data structure `data` and a dot-separated
/// `key_chain` that specifies the path to a nested field. It attempts to retrieve
/// the value of the specified field. If the field is found, it is returned as a
/// `Value`. If any part of the key chain is not found, an error is returned.
///
/// # Arguments
///
/// * `data` - The serializable data structure to retrieve the field from.
/// * `key_chain` - A dot-separated string that specifies the path to the nested field.
///
/// # Returns
///
/// A `Result` containing the value of the specified nested field, or an error if
/// any part of the key chain is not found or the field cannot be deserialized.
pub fn get_nested_value<T, R>(data: T, key_chain: &str) -> Result<R>
where
    T: Serialize,
    R: DeserializeOwned,
{
    let parts: VecDeque<&str> = key_chain.split('.').collect();
    let mut current_value = serde_value::to_value(data).unwrap();

    for key in parts {
        match current_value {
            Value::Map(mut map) => {
                let value_key = Value::String(key.to_owned());
                current_value = map.remove(&value_key).ok_or_else(|| {
                    Error::new(ErrorKind::NotFound, format!("Key '{}' not found", key))
                })?;
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Expected a nested structure",
                ))
            }
        }
    }

    match R::deserialize(current_value) {
        Ok(r) => Ok(r),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e.to_string())),
    }
}
