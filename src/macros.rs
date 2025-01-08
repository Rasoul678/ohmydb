#[macro_export]
/// A macro that generates a struct with the specified fields and implements common traits for it.
///
/// This macro takes a struct name and a list of field names and types, and generates a struct
/// with those fields. It also implements the `Debug`, `Serialize`, `Deserialize`, `Clone`,
/// `PartialEq`, `Eq`, and `Hash` traits for the generated struct.
macro_rules! derive_for_struct {
    ($name:ident, {$($field:ident : $type:ty),*}) => {
        #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
        struct $name {
            $($field: $type),*
        }
    };
}

#[macro_export]
/// A macro that generates a `Display` implementation for a struct, with colored output.
///
/// This macro takes a struct type and a list of its fields, and generates an implementation
/// of the `Display` trait for that struct. The output will be formatted with colored text,
/// using the `colored` crate.
///
/// Each field will be displayed on a new line, with the field name in bright yellow and
/// the field value in bright cyan. The entire output will be enclosed in bright green
/// curly braces.

#[deprecated(since = "2.1.1", note = "Use `display_object` instead")]
macro_rules! display_colored {
    ($t:ty , {$($field:ident: $value:ty),*}) => {

        impl Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}\n", "{".bright_green().bold(),)?;
                $(write!(
                    f,
                    "   {}: {:?},\n",
                    stringify!($field).bright_yellow().bold(),
                    self.$field

                )?;)*
                write!(
                    f,
                    " {}",
                    "}".bright_green().bold()
                )
            }
        }
    };
}

#[macro_export]
/// A macro that generates a struct from a list of field names and types.
///
/// This macro takes a struct name and a list of field names and types, and generates
/// a new struct with those fields. It also implements the `Debug`, `Serialize`,
/// `Deserialize`, `Clone`, `PartialEq`, `Eq`, and `Hash` traits for the generated struct.
/// Additionally, it generates a `Display` implementation for the struct that formats the
/// output with colored text using the `display_colored` macro.
macro_rules! define_struct_from {
    ($($t:ident {$($field:ident: $type:ty),*}),* ) => {
        use std::fmt::Display;
        use $crate::serde::{Deserialize, Serialize};
        use $crate::colored::Colorize;
        use $crate::derive_for_struct;

        $(
            derive_for_struct!($t, {$($field: $type),* });
            // display_colored!($t, {$($field: $type),* });
        )*
    };
}
