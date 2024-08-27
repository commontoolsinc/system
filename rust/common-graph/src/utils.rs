use std::collections::HashSet;

/// Whether this tuple iterator of an optional 2nd field
/// contains all `Some` values.
pub fn is_full<'a, I, K, V>(list: I) -> bool
where
    K: 'a,
    V: 'a,
    I: IntoIterator<Item = &'a (K, Option<V>)>,
{
    list.into_iter().all(|(_, v)| v.is_some())
}

/// Whether this tuple iterator of an optional 2nd field
/// contains all `None` values.
pub fn is_empty<'a, I, K, V>(list: I) -> bool
where
    K: 'a,
    V: 'a,
    I: IntoIterator<Item = &'a (K, Option<V>)>,
{
    list.into_iter().all(|(_, v)| v.is_none())
}

/// Returns `None` if all entries in `list` are unique.
/// Otherwise returns a `Some` value of a duplicated entry.
pub fn non_unique_entries<I, V>(list: I) -> Option<V>
where
    V: Copy + std::hash::Hash + std::cmp::Eq,
    I: IntoIterator<Item = V>,
{
    let mut set: HashSet<V> = HashSet::default();
    list.into_iter().find(|&item| !set.insert(item))
}
