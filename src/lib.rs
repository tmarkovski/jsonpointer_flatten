//! Rust library to flatten a JSON object using JSON Pointer field addressing as defined in [IETF RFC 6901](https://datatracker.ietf.org/doc/html/rfc6901).
use serde::Serialize;
use serde_json::{json, Map, Result, Value};

type PointerMap = Vec<String>;
type ValueMap = Map<String, Value>;

/// Flatten a JSON string
///
/// # Example
///
/// ```
/// let value = r#"
/// {
///     "name": "John Smith",
///     "age": 24,
///     "address": {
///         "country": "US",
///         "zip": "00000"
///     },
///     "phones": [ "123", "456" ]
/// }
/// "#;
///
/// let result = jsonpointer_flatten::from_str(&value);
/// ```
pub fn from_str(s: &str) -> Result<Value> {
    Ok(from_json(&serde_json::from_str::<Value>(s)?))
}

/// Flatten a JSON value
///
/// # Example
///
/// ```
/// use serde_json::json;
///
/// let value = json!(
/// {
///     "name": "John Smith",
///     "age": 24,
///     "address": {
///         "country": "US",
///         "zip": "00000"
///     },
///     "phones": [ "123", "456" ]
/// }
/// );
///
/// let result = jsonpointer_flatten::from_json(&value);
/// ```
pub fn from_json(value: &Value) -> Value {
    let mut route = PointerMap::new();
    let mut target = ValueMap::new();

    process(&value, &mut route, &mut target);

    Value::Object(target)
}

/// Flatten a struct value
///
/// # Example
///
/// ```
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Person<'a> {
///     name: &'a str,
///     age: u8
/// }
///
/// let value = Person {
///     name: "John Smith",
///     age: 24
/// };
///
/// let result = jsonpointer_flatten::from(&value);
/// ```
pub fn from<T>(value: &T) -> Result<Value>
where
    T: Serialize,
{
    from_str(&serde_json::to_string(value)?)
}

fn process(value: &Value, route: &mut PointerMap, target: &mut ValueMap) {
    match value {
        Value::Null => {
            target.insert(route.concat(), Value::Null);
        }
        Value::Bool(b) => {
            target.insert(route.concat(), Value::Bool(b.clone()));
        }
        Value::Number(n) => {
            target.insert(route.concat(), Value::Number(n.clone()));
        }
        Value::String(s) => {
            target.insert(route.concat(), Value::String(s.clone()));
        }
        Value::Array(arr) => {
            target.insert(route.concat(), json!([]));
            arr.iter().enumerate().for_each(|(idx, val)| {
                route.push(format!("/{}", idx));
                process(val, route, target);
            });
        }
        Value::Object(obj) => {
            target.insert(route.concat(), json!({}));
            for (key, val) in obj {
                route.push(format!("/{}", escape(key.as_str())));
                process(val, route, target);
            }
        }
    }
    route.pop();
}

fn escape<'a>(value: &'a str) -> String {
    value.replace("~", "~0").replace("/", "~1")
}

#[allow(dead_code)]
fn unescape<'a>(value: &'a str) -> String {
    value.replace("~1", "/").replace("~0", "~")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serialize() {
        let result = from_str("{ \"one\": 1 }").unwrap();

        assert!(result.is_object());
        assert!(result.as_object().unwrap().len() == 2);
    }

    #[test]
    fn test_spec_values() {
        let value = json!(
            {
                "foo": ["bar", "baz"],
                "": 0,
                "a/b": 1,
                "c%d": 2,
                "e^f": 3,
                "g|h": 4,
                "i\\j": 5,
                "k\"l": 6,
                " ": 7,
                "m~n": 8
             }
        );

        let actual = from_json(&value);

        assert!(actual.is_object());

        assert!(actual.get("/m~0n").unwrap().eq(&json!(8)));
        assert!(actual.get("/a~1b").unwrap().eq(&json!(1)));
        assert!(actual.get("/ ").unwrap().eq(&json!(7)));
    }

    #[test]
    fn flatten_top_level_array() {
        let value = json!([true, 42]);

        let actual = from(&value).unwrap();

        assert!(actual.get("").unwrap().eq(&json!([])));
        assert!(actual.get("/0").unwrap().eq(&json!(true)));
        assert!(actual.get("/1").unwrap().eq(&json!(42)));
    }

    #[test]
    fn test_readme_example() {
        let value = json!(
            {
                "name": "John Smith",
                "age": 24,
                "address": {
                    "country": "US",
                    "zip": "00000"
                },
                "phones": [ "123", "456" ]
            }
        );

        let result = from_json(&value);

        assert!(result.is_object());
    }

    #[test]
    fn test_null() {
        let value = json!({ "name": null });

        let result = from_json(&value);

        assert!(result.is_object());
        assert!(result
            .as_object()
            .unwrap()
            .get("/name")
            .unwrap()
            .eq(&json!(Value::Null)));
    }

    #[test]
    fn flatten_array() {
        let value = json!(
            [
                1,
                "name",
                {
                    "country": "US",
                    "zip": "00000"
                },
                [ "123", "456" ]
            ]
        );

        let result = from_json(&value);

        assert!(result.is_object());
    }

    #[test]
    fn flatten_values() {
        let value = json!(42);

        let result = from_json(&value);

        assert!(result.is_object());
    }

    #[test]
    fn flatten_from_str_throws_invalid_json() {
        let value = "not json";

        let result = from_str(&value);

        assert!(result.is_err());
    }

    #[test]
    fn flatten_from_custom_type() {
        let value = Person {
            name: "John Smith",
            age: 24,
        };

        let result = from(&value);

        assert!(result.is_ok());
        assert!(result.unwrap().is_object());
    }

    #[derive(Serialize)]
    struct Person<'a> {
        name: &'a str,
        age: u8,
    }
}
