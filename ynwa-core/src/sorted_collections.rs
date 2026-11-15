//! Deterministic serialization of unordered collections.
//!
//! `HashMap`/`HashSet` iteration order differs between processes, so values are written through
//! their ordered counterparts. Deserialization needs no counterpart — any order is accepted.

use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

pub fn map<K, V, S>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    K: Ord + Serialize,
    V: Serialize,
    S: Serializer,
{
    BTreeMap::from_iter(value.iter()).serialize(serializer)
}

pub fn set<S: Serializer>(value: &HashSet<String>, serializer: S) -> Result<S::Ok, S::Error> {
    BTreeSet::from_iter(value.iter()).serialize(serializer)
}
