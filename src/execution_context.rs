//! Ordered JSON variable bindings carried through workflow execution.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Execution context holding variable bindings and state.
///
/// Uses IndexMap to preserve insertion order of variables. Values can be any JSON type
/// or None (unset).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    #[serde(flatten)]
    variables: IndexMap<String, Option<Value>>,
}

impl ExecutionContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        ExecutionContext {
            variables: IndexMap::new(),
        }
    }

    /// Set a variable to a JSON value.
    pub fn set<S: Into<String>>(&mut self, name: S, value: Value) {
        self.variables.insert(name.into(), Some(value));
    }

    /// Set a variable to None (unset).
    pub fn unset<S: Into<String>>(&mut self, name: S) {
        self.variables.insert(name.into(), None);
    }

    /// Get a variable's value.
    pub fn get<S: AsRef<str>>(&self, name: S) -> Option<&Value> {
        self.variables.get(name.as_ref()).and_then(|v| v.as_ref())
    }

    /// Check if a variable is set (not None).
    pub fn contains<S: AsRef<str>>(&self, name: S) -> bool {
        self.variables
            .get(name.as_ref())
            .map(|v| v.is_some())
            .unwrap_or(false)
    }

    /// Remove a variable entirely.
    pub fn remove<S: AsRef<str>>(&mut self, name: S) -> Option<Option<Value>> {
        self.variables.shift_remove(name.as_ref())
    }

    /// Get all variables as a reference to the underlying map.
    pub fn all(&self) -> &IndexMap<String, Option<Value>> {
        &self.variables
    }

    /// Get a mutable reference to all variables.
    pub fn all_mut(&mut self) -> &mut IndexMap<String, Option<Value>> {
        &mut self.variables
    }

    /// Clear all variables.
    pub fn clear(&mut self) {
        self.variables.clear();
    }

    /// Get the number of variables.
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    /// Serialize the context to a JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Create context from a map.
    pub fn from_map(map: IndexMap<String, Option<Value>>) -> Self {
        ExecutionContext { variables: map }
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get() {
        let mut ctx = ExecutionContext::new();
        ctx.set("name", Value::String("Alice".to_string()));
        assert_eq!(ctx.get("name"), Some(&Value::String("Alice".to_string())));
    }

    #[test]
    fn test_unset() {
        let mut ctx = ExecutionContext::new();
        ctx.set("key", Value::String("value".to_string()));
        ctx.unset("key");
        assert!(!ctx.contains("key"));
    }

    #[test]
    fn test_remove() {
        let mut ctx = ExecutionContext::new();
        ctx.set("key", Value::Number(42.into()));
        let removed = ctx.remove("key");
        assert_eq!(removed, Some(Some(Value::Number(42.into()))));
        assert_eq!(ctx.get("key"), None);
    }

    #[test]
    fn test_contains() {
        let mut ctx = ExecutionContext::new();
        ctx.set("set_var", Value::Bool(true));
        ctx.unset("unset_var");

        assert!(ctx.contains("set_var"));
        assert!(!ctx.contains("unset_var"));
        assert!(!ctx.contains("unknown_var"));
    }

    #[test]
    fn test_json_number() {
        let mut ctx = ExecutionContext::new();
        ctx.set("count", Value::Number(42.into()));
        assert_eq!(ctx.get("count"), Some(&Value::Number(42.into())));
    }

    #[test]
    fn test_json_array() {
        let mut ctx = ExecutionContext::new();
        let arr = Value::Array(vec![Value::String("a".into()), Value::String("b".into())]);
        ctx.set("items", arr.clone());
        assert_eq!(ctx.get("items"), Some(&arr));
    }

    #[test]
    fn test_json_object() {
        let mut ctx = ExecutionContext::new();
        let obj = serde_json::json!({"x": 1, "y": 2});
        ctx.set("point", obj.clone());
        assert_eq!(ctx.get("point"), Some(&obj));
    }

    #[test]
    fn test_clear() {
        let mut ctx = ExecutionContext::new();
        ctx.set("a", Value::Null);
        ctx.set("b", Value::Bool(false));
        assert!(!ctx.is_empty());
        ctx.clear();
        assert!(ctx.is_empty());
    }

    #[test]
    fn test_insertion_order() {
        let mut ctx = ExecutionContext::new();
        ctx.set("z", Value::Null);
        ctx.set("a", Value::Null);
        ctx.set("m", Value::Null);

        let keys: Vec<_> = ctx.all().keys().collect();
        assert_eq!(keys, vec!["z", "a", "m"]);
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut ctx = ExecutionContext::new();
        ctx.set("name", Value::String("Bob".to_string()));
        ctx.set("age", Value::Number(30.into()));
        ctx.unset("unset_field");

        let json = serde_json::to_string(&ctx).unwrap();
        let restored: ExecutionContext = serde_json::from_str(&json).unwrap();

        assert_eq!(
            restored.get("name"),
            Some(&Value::String("Bob".to_string()))
        );
        assert_eq!(restored.get("age"), Some(&Value::Number(30.into())));
        assert!(!restored.contains("unset_field"));
    }
}
