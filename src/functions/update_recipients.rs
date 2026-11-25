use crate::{FNOX_CONFIG, TEST_PROFILE, get_toml_item_mut, push_unique_toml_string};
use age::x25519::Identity;
use errgonomic::handle;
use thiserror::Error;
use toml_edit::{DocumentMut, Item};
use xshell::Shell;

pub fn update_recipients(sh: &Shell, identity: &Identity) -> Result<(), UpdateRecipientsError> {
    use UpdateRecipientsError::*;
    let recipient = identity.to_public().to_string();
    let contents = handle!(sh.read_file(FNOX_CONFIG), ReadFileFailed);
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
        handle!(sh.write_file(FNOX_CONFIG, doc.to_string()), WriteFileFailed);
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum UpdateRecipientsError {
    #[error("failed to read '{FNOX_CONFIG}'")]
    ReadFileFailed { source: xshell::Error },
    #[error("failed to parse '{FNOX_CONFIG}'")]
    ParseFailed { source: Box<toml_edit::TomlError>, contents: String },
    #[error("failed to write '{FNOX_CONFIG}'")]
    WriteFileFailed { source: xshell::Error },
}
