use std::fmt::Display;

use serde_json::{Map, Value};

use crate::types::Context;

#[derive(Debug, Clone)]
pub enum ConditionOperator {
    Is,
    In,
    Has,
    Between,
    Other(String),
}

impl Display for ConditionOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Has => f.write_str("has"),
            Self::Is => f.write_str("is"),
            Self::In => f.write_str("in"),
            Self::Between => f.write_str("between"),
            Self::Other(o) => f.write_str(o),
        }
    }
}

impl From<(String, &Vec<Value>)> for ConditionOperator {
    fn from(value: (String, &Vec<Value>)) -> Self {
        let (operator, operands) = value;
        let operand_0 = operands.get(0);
        let operand_1 = operands.get(1);
        let operand_2 = operands.get(2);
        match (operator.as_str(), operand_0, operand_1, operand_2) {
            // assuming there will be only two operands, one with the dimension name and other with the value
            ("==", _, _, None) => ConditionOperator::Is,
            ("<=", Some(_), Some(Value::Object(a)), Some(_)) if a.contains_key("var") => {
                ConditionOperator::Between
            }
            // assuming there will be only two operands, one with the dimension name and other with the value
            ("in", Some(Value::Object(a)), Some(_), None) if a.contains_key("var") => {
                ConditionOperator::In
            }
            // assuming there will be only two operands, one with the dimension name and other with the value
            ("in", Some(_), Some(Value::Object(a)), None) if a.contains_key("var") => {
                ConditionOperator::Has
            }
            _ => ConditionOperator::Other(operator),
        }
    }
}

#[derive(Clone)]
pub struct Condition {
    pub left_operand: String,
    pub operator: ConditionOperator,
    pub right_operand: String,
}

impl TryFrom<&Map<String, Value>> for Condition {
    type Error = &'static str;
    fn try_from(source: &Map<String, Value>) -> Result<Self, Self::Error> {
        if let Some(operator) = source.keys().next() {
            let emty_vec = vec![];
            let operands = source[operator].as_array().unwrap_or(&emty_vec);

            let operator = ConditionOperator::from((operator.to_owned(), operands));

            let dimension_name = operands
                .iter()
                .find_map(|item| match item.as_object() {
                    Some(o) if o.contains_key("var") => {
                        Some(o["var"].as_str().unwrap_or(""))
                    }
                    _ => None,
                })
                .unwrap_or("");

            let other_operands = operands
                .iter()
                .filter_map(|item| {
                    if item.is_object() && item.as_object().unwrap().contains_key("var") {
                        return None;
                    }

                    match item {
                        Value::Null => String::from("null"),
                        Value::String(v) => v.clone(),
                        _ => format!("{}", item),
                    }
                    .into()
                })
                .collect::<Vec<String>>()
                .join(",");

            return Ok(Condition {
                operator,
                left_operand: dimension_name.to_owned(),
                right_operand: other_operands,
            });
        }

        Err("not a valid condition map")
    }
}

impl TryFrom<&Value> for Condition {
    type Error = &'static str;
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let obj = value
            .as_object()
            .ok_or("not a valid condition value, should be an object")?;
        Condition::try_from(obj)
    }
}

impl TryFrom<&Context> for Vec<Condition> {
    type Error = &'static str;
    fn try_from(context: &Context) -> Result<Self, Self::Error> {
        context
            .condition
            .as_object()
            .ok_or("failed to parse context.condition as an object")
            .and_then(|obj| match obj.get("and") {
                Some(v) => v
                    .as_array()
                    .ok_or("failed to parse value of and as array")
                    .and_then(|arr| {
                        arr.iter()
                            .map(|condition| Condition::try_from(condition))
                            .collect::<Result<Vec<Condition>, &'static str>>()
                    }),
                None => Condition::try_from(obj).and_then(|v| Ok(vec![v])),
            })
    }
}

impl Into<String> for Condition {
    fn into(self) -> String {
        format!(
            "{} {} {}",
            self.left_operand, self.operator, self.right_operand
        )
    }
}
