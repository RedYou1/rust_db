use std::io;

pub fn remove_errors<T>(items: impl Iterator<Item = io::Result<T>>) -> io::Result<Vec<T>> {
    let mut result = vec![];
    for (i, item) in items.enumerate() {
        match item {
            Ok(item) => result.insert(i, item),
            Err(err) => return Err(err),
        }
    }
    Ok(result)
}

pub fn flat_remove_errors<T>(
    items: impl Iterator<Item = io::Result<Vec<T>>>,
) -> io::Result<Vec<T>> {
    let mut result = vec![];
    for (i, item) in items.enumerate() {
        match item {
            Ok(item) => result.insert(i, item),
            Err(err) => return Err(err),
        }
    }
    Ok(result.into_iter().flatten().collect())
}
