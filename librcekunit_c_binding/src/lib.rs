#![forbid(unsafe_op_in_unsafe_fn)]
#![allow(clippy::missing_safety_doc)]

mod buffer;
mod client;
mod error;

pub use buffer::{lr_bytes_free, lr_string_free};
pub use client::{
    LrClient, lr_client_delete_all, lr_client_delete_by_category, lr_client_destroy,
    lr_client_export, lr_client_free, lr_client_import_csv, lr_client_index,
    lr_client_input_user_export, lr_client_login, lr_client_logout, lr_client_new, lr_client_show,
    lr_client_store, lr_client_update, lr_config_debug_string,
};
pub use error::LrStatus;
