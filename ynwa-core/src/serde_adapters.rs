//! Serde adapters that make records unit-explicit and byte-deterministic.
//!
//! - uom quantities are written as plain `f32` in canonical units (meters, meters per second,
//!   degrees) rather than in their internal base representation, so a record can be read without
//!   knowing uom internals;
//! - unordered collections are written in a stable (sorted) order, so equal values always
//!   produce equal bytes regardless of `HashMap`/`HashSet` iteration order.
//!
//! Guarantee (v1): determinism holds within a single platform; cross-platform float formatting
//! is out of scope (see `task_6_serialization_crossplatform`).

pub mod meters {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use uom::si::f32::Length;
    use uom::si::length::meter;

    pub fn serialize<S: Serializer>(value: &Length, serializer: S) -> Result<S::Ok, S::Error> {
        value.get::<meter>().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Length, D::Error> {
        Ok(Length::new::<meter>(f32::deserialize(deserializer)?))
    }
}

pub mod meters_per_second {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use uom::si::f32::Velocity;
    use uom::si::velocity::meter_per_second;

    pub fn serialize<S: Serializer>(value: &Velocity, serializer: S) -> Result<S::Ok, S::Error> {
        value.get::<meter_per_second>().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Velocity, D::Error> {
        Ok(Velocity::new::<meter_per_second>(f32::deserialize(
            deserializer,
        )?))
    }
}

pub mod degrees {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use uom::si::angle::degree;
    use uom::si::f32::Angle;

    pub fn serialize<S: Serializer>(value: &Angle, serializer: S) -> Result<S::Ok, S::Error> {
        value.get::<degree>().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Angle, D::Error> {
        Ok(Angle::new::<degree>(f32::deserialize(deserializer)?))
    }
}

/// Writes a `HashMap<String, V>` as a JSON object with keys in lexicographic order.
pub mod sorted_map {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::{BTreeMap, HashMap};

    pub fn serialize<S, V>(map: &HashMap<String, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        V: Serialize,
    {
        BTreeMap::from_iter(map.iter()).serialize(serializer)
    }

    pub fn deserialize<'de, D, V>(deserializer: D) -> Result<HashMap<String, V>, D::Error>
    where
        D: Deserializer<'de>,
        V: Deserialize<'de>,
    {
        Ok(BTreeMap::<String, V>::deserialize(deserializer)?
            .into_iter()
            .collect())
    }
}

/// Writes a `HashSet<String>` as an array with values in lexicographic order.
pub mod sorted_set {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::{BTreeSet, HashSet};

    pub fn serialize<S: Serializer>(
        set: &HashSet<String>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        BTreeSet::from_iter(set.iter()).serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<HashSet<String>, D::Error> {
        Ok(BTreeSet::<String>::deserialize(deserializer)?
            .into_iter()
            .collect())
    }
}
