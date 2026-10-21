use serde_json::{Value, json};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    Call {
        id: String,
        action: String,
        payload: Value,
    },
    CallResult {
        id: String,
        payload: Value,
    },
    CallError {
        id: String,
        code: String,
        description: String,
        details: Value,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum FrameError {
    NotJson,
    NotArray,
    UnknownType,
    WrongLength,
    BadField(&'static str),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FrameError::NotJson => write!(f, "frame is not valid JSON"),
            FrameError::NotArray => write!(f, "frame is not a JSON array"),
            FrameError::UnknownType => write!(f, "unknown message type id"),
            FrameError::WrongLength => write!(f, "wrong number of elements for message type"),
            FrameError::BadField(name) => write!(f, "bad field: {name}"),
        }
    }
}

impl std::error::Error for FrameError {}

pub fn parse(text: &str) -> Result<Frame, FrameError> {
    let value: Value = serde_json::from_str(text).map_err(|_| FrameError::NotJson)?;
    let Value::Array(items) = value else {
        return Err(FrameError::NotArray);
    };
    let type_id = items.first().and_then(Value::as_u64);
    let length = match type_id {
        Some(2) => 4,
        Some(3) => 3,
        Some(4) => 5,
        _ => return Err(FrameError::UnknownType),
    };
    if items.len() != length {
        return Err(FrameError::WrongLength);
    }
    let id = string(&items[1], "id")?;
    match type_id {
        Some(2) => Ok(Frame::Call {
            id,
            action: string(&items[2], "action")?,
            payload: items[3].clone(),
        }),
        Some(3) => Ok(Frame::CallResult {
            id,
            payload: items[2].clone(),
        }),
        _ => Ok(Frame::CallError {
            id,
            code: string(&items[2], "code")?,
            description: string(&items[3], "description")?,
            details: object(&items[4], "details")?,
        }),
    }
}

pub fn serialize(frame: &Frame) -> String {
    let value = match frame {
        Frame::Call {
            id,
            action,
            payload,
        } => json!([2, id, action, payload]),
        Frame::CallResult { id, payload } => json!([3, id, payload]),
        Frame::CallError {
            id,
            code,
            description,
            details,
        } => json!([4, id, code, description, details]),
    };
    value.to_string()
}

fn string(value: &Value, field: &'static str) -> Result<String, FrameError> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or(FrameError::BadField(field))
}

fn object(value: &Value, field: &'static str) -> Result<Value, FrameError> {
    if value.is_object() {
        Ok(value.clone())
    } else {
        Err(FrameError::BadField(field))
    }
}

#[cfg(test)]
#[path = "frame.test.rs"]
mod tests;
