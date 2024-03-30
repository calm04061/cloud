use rbatis::rbdc::DateTime;
use rbs::Value;

use crate::{EventMessage, EventResult, EventType};

impl EventMessage {
    pub fn sync_default() -> Value {
        let map = rbs::to_value! {
            "id":"INTEGER PRIMARY KEY AUTOINCREMENT",
            "event_type":"int not null",
            "event_result":"text not null",
            "message":"int8 not null",
            "create_time":"int8 not null",
            };
        map
    }
    fn new(event_type: EventType, result: EventResult, message: String) -> EventMessage {
        let event_message = EventMessage {
            id: None,
            event_type: event_type.into(),
            event_result: result.into(),
            message,
            create_time: DateTime::now(),

        };
        event_message
    }
    pub fn success(event_type: EventType, message: String) -> EventMessage {
        EventMessage::new(event_type, EventResult::Success, message)
    }
    pub fn fail(event_type: EventType, message: String) -> EventMessage {
        EventMessage::new(event_type, EventResult::Fail, message)
    }
}

impl From<EventType> for i8 {
    fn from(value: EventType) -> Self {
        match value {
            EventType::UploadFileBlock => 1,
        }
    }
}

impl From<EventResult> for i8 {
    fn from(value: EventResult) -> Self {
        match value {
            EventResult::Fail => 0,
            EventResult::Success => 1,
        }
    }
}