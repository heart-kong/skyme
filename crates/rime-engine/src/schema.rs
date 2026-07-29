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

#[cfg(test)]
mod tests {
    use crate::schema::SchemaInfo;

    #[test]
    fn test_schema_info() {
        let s = SchemaInfo::new("luna_pinyin", "朙月拼音");
        assert_eq!(s.schema_id, "luna_pinyin");
        assert_eq!(s.name, "朙月拼音");
    }

    #[test]
    fn test_schema_info_from_string() {
        let s = SchemaInfo::new(String::from("terra_pinyin"), String::from("地球拼音"));
        assert_eq!(s.schema_id, "terra_pinyin");
    }
}
