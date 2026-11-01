use std::collections::{HashMap, HashSet};

use crate::error::{FigError, Result};
use crate::limits::Limits;

use super::cursor::{Cursor, CursorError};

pub(crate) const TYPE_BOOL: i32 = -1;
pub(crate) const TYPE_BYTE: i32 = -2;
pub(crate) const TYPE_INT: i32 = -3;
pub(crate) const TYPE_UINT: i32 = -4;
pub(crate) const TYPE_FLOAT: i32 = -5;
pub(crate) const TYPE_STRING: i32 = -6;
pub(crate) const TYPE_INT64: i32 = -7;
pub(crate) const TYPE_UINT64: i32 = -8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DefKind {
    Enum,
    Struct,
    Message,
}

#[derive(Debug, Clone)]
pub(crate) struct Field {
    pub name: String,
    pub type_id: i32,
    pub is_array: bool,
    pub value: u32,
}

#[derive(Debug)]
pub(crate) struct Def {
    pub name: String,
    pub kind: DefKind,
    pub fields: Vec<Field>,
    by_value: HashMap<u32, usize>,
}

impl Def {
    fn new(name: String, kind: DefKind, fields: Vec<Field>) -> Self {
        let mut by_value = HashMap::with_capacity(fields.len());
        for (index, field) in fields.iter().enumerate() {
            by_value.insert(field.value, index);
        }
        Self {
            name,
            kind,
            fields,
            by_value,
        }
    }

    pub(crate) fn field_by_value(&self, value: u32) -> Option<&Field> {
        self.by_value.get(&value).map(|index| &self.fields[*index])
    }
}

#[derive(Debug)]
pub(crate) struct Schema {
    pub defs: Vec<Def>,
    by_name: HashMap<String, usize>,
}

impl Schema {
    /// Parse the binary Kiwi schema with hard bounds on definitions, fields,
    /// names, and type references.
    pub(crate) fn parse(bytes: &[u8], limits: &Limits) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let def_count = cursor
            .read_var_uint()
            .map_err(|error| schema_error("definition count", error))?
            as usize;

        if def_count == 0 {
            return Err(FigError::Schema("schema has zero definitions".into()));
        }
        if def_count > limits.max_schema_defs {
            return Err(FigError::limit(
                "schema definition count",
                def_count,
                limits.max_schema_defs,
            ));
        }
        // Each definition needs at least a NUL-terminated name, a kind byte, and a
        // field count: 3 bytes. Reject absurd counts before looping.
        if def_count > cursor.remaining() / 3 + 1 {
            return Err(FigError::Schema(
                "definition count is larger than the remaining schema bytes can hold".into(),
            ));
        }

        let mut defs = Vec::new();
        let mut total_fields = 0usize;
        for _ in 0..def_count {
            let name = read_name(&mut cursor, limits, "definition name")?;
            let kind = match cursor
                .read_u8()
                .map_err(|error| schema_error("definition kind", error))?
            {
                0 => DefKind::Enum,
                1 => DefKind::Struct,
                2 => DefKind::Message,
                other => {
                    return Err(FigError::Schema(format!(
                        "definition {name:?} has unknown kind byte {other}"
                    )))
                }
            };
            let field_count = cursor
                .read_var_uint()
                .map_err(|error| schema_error("field count", error))?
                as usize;
            total_fields = total_fields
                .checked_add(field_count)
                .ok_or_else(|| FigError::Schema("field count overflow".into()))?;
            if total_fields > limits.max_schema_fields {
                return Err(FigError::limit(
                    "schema field count",
                    total_fields,
                    limits.max_schema_fields,
                ));
            }
            if field_count > cursor.remaining() / 4 + 1 {
                return Err(FigError::Schema(format!(
                    "definition {name:?} declares more fields than the remaining bytes can hold"
                )));
            }

            let mut fields = Vec::new();
            let mut values = HashSet::new();
            let mut names = HashSet::new();
            for _ in 0..field_count {
                let field_name = read_name(&mut cursor, limits, "field name")?;
                if !names.insert(field_name.clone()) {
                    return Err(FigError::Schema(format!(
                        "definition {name:?} has duplicate field name {field_name:?}"
                    )));
                }
                let type_id = cursor
                    .read_var_int()
                    .map_err(|error| schema_error("field type", error))?;
                let is_array = cursor
                    .read_bool()
                    .map_err(|error| schema_error("field array flag", error))?;
                let value = cursor
                    .read_var_uint()
                    .map_err(|error| schema_error("field id", error))?;

                validate_type_id(type_id, def_count, &name, &field_name)?;
                if matches!(kind, DefKind::Message | DefKind::Enum) && !values.insert(value) {
                    return Err(FigError::Schema(format!(
                        "definition {name:?} has duplicate field id {value}"
                    )));
                }
                fields.push(Field {
                    name: field_name,
                    type_id,
                    is_array,
                    value,
                });
            }
            defs.push(Def::new(name, kind, fields));
        }

        if !cursor.is_empty() {
            return Err(FigError::Schema(format!(
                "schema has {} trailing bytes",
                cursor.remaining()
            )));
        }

        let mut by_name = HashMap::with_capacity(defs.len());
        for (index, def) in defs.iter().enumerate() {
            if by_name.insert(def.name.clone(), index).is_some() {
                return Err(FigError::Schema(format!(
                    "duplicate definition name {:?}",
                    def.name
                )));
            }
        }

        Ok(Self { defs, by_name })
    }

    pub(crate) fn index_of(&self, name: &str) -> Option<usize> {
        self.by_name.get(name).copied()
    }

    pub(crate) fn def(&self, index: usize) -> Option<&Def> {
        self.defs.get(index)
    }
}

fn validate_type_id(type_id: i32, def_count: usize, def: &str, field: &str) -> Result<()> {
    let builtin = (TYPE_UINT64..=TYPE_BOOL).contains(&type_id);
    let user_type = type_id >= 0 && (type_id as usize) < def_count;
    if builtin || user_type {
        Ok(())
    } else {
        Err(FigError::Schema(format!(
            "field {def}.{field} references invalid type {type_id}; schema has {def_count} definitions"
        )))
    }
}

fn read_name(cursor: &mut Cursor<'_>, limits: &Limits, what: &str) -> Result<String> {
    let name = cursor
        .read_string(limits.max_schema_name_bytes)
        .map_err(|error| schema_error(what, error))?;
    if name.is_empty() {
        return Err(FigError::Schema(format!("{what} is empty")));
    }
    Ok(name.to_owned())
}

fn schema_error(context: &str, error: CursorError) -> FigError {
    FigError::Schema(format!("{context}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_limits() -> Limits {
        Limits {
            max_schema_defs: 16,
            max_schema_fields: 64,
            max_schema_name_bytes: 32,
            ..Limits::for_fuzzing()
        }
    }

    #[test]
    fn parses_a_minimal_schema() {
        // message ABC { int[] xyz = 1; }
        let bytes = [1, 65, 66, 67, 0, 2, 1, 120, 121, 122, 0, 5, 1, 1];
        let schema = Schema::parse(&bytes, &tiny_limits()).unwrap();
        assert_eq!(schema.defs.len(), 1);
        assert_eq!(schema.defs[0].name, "ABC");
        assert_eq!(schema.defs[0].kind, DefKind::Message);
        assert_eq!(schema.defs[0].fields.len(), 1);
        assert_eq!(schema.defs[0].fields[0].name, "xyz");
    }

    #[test]
    fn rejects_empty_schema() {
        assert!(Schema::parse(&[0], &tiny_limits()).is_err());
    }

    #[test]
    fn rejects_invalid_type_reference() {
        // One message with a field of type 5 when only one definition exists.
        let bytes = [1, 1, b'A', 2, 1, 1, b'f', 10, 0, 1];
        assert!(Schema::parse(&bytes, &tiny_limits()).is_err());
    }
}
