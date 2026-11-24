use toml_edit::Item;

pub fn get_toml_item_mut(item: &mut Item, path: impl IntoIterator<Item = impl AsRef<str>>) -> Option<&mut Item> {
    path.into_iter()
        .try_fold(item, |item, key| item.as_table_like_mut()?.get_mut(key.as_ref()))
        .filter(|item| !item.is_none())
}
