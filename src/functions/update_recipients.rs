use crate::{FNOX_CONFIG, TEST_PROFILE, get_toml_item_mut, push_unique_toml_string};
use age::x25519::Identity;
use errgonomic::handle;
use std::{fs, io};
use thiserror::Error;
use toml_edit::{DocumentMut, Item};

pub fn update_recipients(identity: &Identity) -> Result<(), UpdateRecipientsError> {
    use UpdateRecipientsError::*;
    let recipient = identity.to_public().to_string();
    let contents = handle!(fs::read_to_string(FNOX_CONFIG), ReadToStringFailed);
    let mut doc: DocumentMut = handle!(contents.parse(), ParseFailed, contents);
    let has_test_age = get_toml_item_mut(doc.as_item_mut(), ["profiles", TEST_PROFILE, "providers", "age"]).is_some();
    let path: &[&str] = if has_test_age {
        &["profiles", TEST_PROFILE, "providers", "age", "recipients"]
    } else {
        &["providers", "age", "recipients"]
    };
    let added = get_toml_item_mut(doc.as_item_mut(), path)
        .and_then(Item::as_array_mut)
        .is_some_and(|recipients| push_unique_toml_string(recipients, &recipient));
    if added {
        handle!(fs::write(FNOX_CONFIG, doc.to_string()), WriteFailed);
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum UpdateRecipientsError {
    #[error("failed to read '{FNOX_CONFIG}'")]
    ReadToStringFailed { source: io::Error },
    #[error("failed to parse '{FNOX_CONFIG}'")]
    ParseFailed { source: Box<toml_edit::TomlError>, contents: String },
    #[error("failed to write '{FNOX_CONFIG}'")]
    WriteFailed { source: io::Error },
}
