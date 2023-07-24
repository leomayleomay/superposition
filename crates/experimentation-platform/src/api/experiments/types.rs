use std::fmt;

use crate::db::models;
use chrono::{DateTime, Utc};
use serde::{
    de::{self, IntoDeserializer},
    Deserialize, Serialize,
};
use serde_json::Value;

use crate::db::models::ExperimentStatusType;

#[derive(Deserialize, Serialize, Clone)]
pub enum VariantType {
    CONTROL,
    EXPERIMENTAL,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Variant {
    pub id: String,
    pub variant_type: VariantType,
    pub overrides: Value,
}

#[derive(Deserialize)]
pub struct ExperimentCreateRequest {
    pub name: String,
    pub override_keys: Vec<String>,
    pub traffic_percentage: i32,

    pub context: Value,
    pub variants: Vec<Variant>,
}

#[derive(Serialize)]
pub struct ExperimentCreateResponse {
    pub message: String,
    pub data: models::Experiment,
}

#[derive(Deserialize, Debug)]
pub struct ListFilters {
    #[serde(deserialize_with = "deserialize_stringified_list")]
    pub status: Vec<ExperimentStatusType>,
    pub from_date: DateTime<Utc>,
    pub to_date: DateTime<Utc>,
    pub page: i64,
    pub count: i64,
}

pub fn deserialize_stringified_list<'de, D, I>(
    deserializer: D,
) -> std::result::Result<Vec<I>, D::Error>
where
    D: de::Deserializer<'de>,
    I: de::DeserializeOwned,
{
    struct StringVecVisitor<I>(std::marker::PhantomData<I>);

    impl<'de, I> de::Visitor<'de> for StringVecVisitor<I>
    where
        I: de::DeserializeOwned,
    {
        type Value = Vec<I>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing a list")
        }

        fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            let mut query_vector = Vec::new();
            for param in v.split(",") {
                let p: I = I::deserialize(param.into_deserializer())?;
                query_vector.push(p);
            }
            Ok(query_vector)
        }
    }

    deserializer.deserialize_any(StringVecVisitor(std::marker::PhantomData::<I>))
}