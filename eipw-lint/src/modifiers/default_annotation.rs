use annotate_snippets::snippet::AnnotationType;
use std::fmt::Debug;

use crate::lints::Context;
use crate::LintSettings;

use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};

use super::{Error, Modifier};

use tsify::Tsify;

// Define the enum representing the AnnotationTypeEnum
#[derive(Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[derive(Debug, Clone, Copy)]
pub enum AnnotationTypeDef {
    Error,
    Warning,
    Info,
    Note,
    Help,
}

// Manual serialization and deserialization for AnnotationTypeDef using string representation
impl Serialize for AnnotationTypeDef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match *self {
            AnnotationTypeDef::Error => "error",
            AnnotationTypeDef::Warning => "warning",
            AnnotationTypeDef::Info => "info",
            AnnotationTypeDef::Note => "note",
            AnnotationTypeDef::Help => "help",
        };
        serializer.serialize_str(s)
    }
}

impl<'de> Deserialize<'de> for AnnotationTypeDef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_str() {
            "error" => Ok(AnnotationTypeDef::Error),
            "warning" => Ok(AnnotationTypeDef::Warning),
            "info" => Ok(AnnotationTypeDef::Info),
            "note" => Ok(AnnotationTypeDef::Note),
            "help" => Ok(AnnotationTypeDef::Help),
            _ => Err(serde::de::Error::custom("Unknown variant")),
        }
    }
}

// Conversion functions between AnnotationType and AnnotationTypeDef
impl From<AnnotationType> for AnnotationTypeDef {
    fn from(item: AnnotationType) -> Self {
        match item {
            AnnotationType::Error => AnnotationTypeDef::Error,
            AnnotationType::Warning => AnnotationTypeDef::Warning,
            AnnotationType::Info => AnnotationTypeDef::Info,
            AnnotationType::Note => AnnotationTypeDef::Note,
            AnnotationType::Help => AnnotationTypeDef::Help,
        }
    }
}

impl From<AnnotationTypeDef> for AnnotationType {
    fn from(item: AnnotationTypeDef) -> Self {
        match item {
            AnnotationTypeDef::Error => AnnotationType::Error,
            AnnotationTypeDef::Warning => AnnotationType::Warning,
            AnnotationTypeDef::Info => AnnotationType::Info,
            AnnotationTypeDef::Note => AnnotationType::Note,
            AnnotationTypeDef::Help => AnnotationType::Help,
        }
    }
}

// Define the struct using the AnnotationType
#[derive(Tsify, Debug, Clone, Copy, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct SetDefaultAnnotation<S> {
    pub name: S,
    pub value: S,

    #[serde(with = "self::annotation_type_def")]
    #[tsify(type = "AnnotationTypeDef")]
    pub annotation_type: AnnotationType,
}

// Implement a module for serializing/deserializing AnnotationType
mod annotation_type_def {
    use super::{AnnotationType, AnnotationTypeDef};
    use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};

    pub fn serialize<S>(value: &AnnotationType, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let def: AnnotationTypeDef = (*value).into();
        def.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<AnnotationType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let def = AnnotationTypeDef::deserialize(deserializer)?;
        Ok(def.into())
    }
}

// Implement the Modifier trait for SetDefaultAnnotation
impl<S> Modifier for SetDefaultAnnotation<S>
where
    S: Debug + AsRef<str>,
{
    fn modify(&self, context: &Context, settings: &mut LintSettings) -> Result<(), Error> {
        let value = match context.preamble().by_name(self.name.as_ref()) {
            Some(v) => v.value().trim(),
            None => return Ok(()),
        };

        if value == self.value.as_ref() {
            settings.default_annotation_type = self.annotation_type;
        }

        Ok(())
    }
}