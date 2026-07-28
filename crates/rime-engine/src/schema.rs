/// Metadata about an installed Rime schema (input method).
#[derive(Clone, Debug)]
pub struct SchemaInfo {
    pub schema_id: String,
    pub name: String,
}

impl SchemaInfo {
    pub fn new(schema_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            schema_id: schema_id.into(),
            name: name.into(),
        }
    }
}

/// The currently active schema for a session.
#[derive(Clone, Debug)]
pub struct ActiveSchema {
    pub info: SchemaInfo,
}
