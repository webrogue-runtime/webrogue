use std::usize;

#[derive(Clone)]
pub struct Event {
    pub name: &'static str,
    pub id: usize,
    pub fields: Vec<Field>,
    pub size: usize,
}

#[derive(Clone)]
pub struct Field {
    pub name: &'static str,
    pub offset: usize,
    pub ty: FieldType,
}

#[derive(Clone, Copy)]
pub enum FieldType {
    U32,
    Bool,
    U8,
}
impl FieldType {
    pub fn size(&self) -> usize {
        match self {
            FieldType::U32 => 4,
            FieldType::Bool => 1,
            FieldType::U8 => 1,
        }
    }

    pub fn c_name(&self) -> &'static str {
        match self {
            FieldType::U32 => "uint32_t",
            FieldType::Bool => "uint8_t",
            FieldType::U8 => "uint8_t",
        }
    }
}

struct EventsBuilder {
    events: Vec<Event>,
}

impl EventsBuilder {
    fn new() -> Self {
        Self { events: Vec::new() }
    }

    fn event<'a>(&'a mut self, name: &'static str, id: usize) -> EventBuilder<'a> {
        self.events.push(Event {
            name,
            id,
            fields: Vec::new(),
            size: 4,
        });
        EventBuilder {
            event: self.events.last_mut().unwrap(),
        }
    }

    fn events(self) -> Vec<Event> {
        self.events
    }
}

struct EventBuilder<'a> {
    event: &'a mut Event,
}

impl EventBuilder<'_> {
    fn field(&mut self, name: &'static str, ty: FieldType) -> &mut Self {
        self.event.fields.push(Field {
            name,
            offset: self.event.size,
            ty,
        });
        self.align();
        self
    }

    fn align(&mut self) {
        let mut field_refs = self.event.fields.iter_mut().collect::<Vec<_>>();
        field_refs.sort_by_key(|field| usize::MAX - field.ty.size());
        self.event.size = 4;
        for field in &mut field_refs {
            field.offset = self.event.size;
            self.event.size += field.ty.size();
        }
        self.event.size = ((self.event.size + 3) / 4) * 4;
    }
}

pub fn all() -> Vec<Event> {
    use FieldType::*;

    let mut builder = EventsBuilder::new();

    builder
        .event("mouse_button", 1)
        .field("button", U32)
        .field("down", Bool)
        .field("x", U32)
        .field("y", U32);

    builder
        .event("mouse_motion", 2)
        .field("x", U32)
        .field("y", U32);

    builder
        .event("key", 3)
        .field("down", Bool)
        .field("scancode", U32);

    builder.event("quit", 4);

    builder.event("window_resized", 5);
    builder.event("gl_resized", 6);

    builder.event("text_input", 7).field("c", U8);

    // This value must be incremented when an event is added or changed,
    // then event's id must be changed to this value:
    // 7

    builder.events()
}
