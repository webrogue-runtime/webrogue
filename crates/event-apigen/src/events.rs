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
}
impl FieldType {
    pub fn size(&self) -> usize {
        match self {
            FieldType::U32 => 4,
        }
    }

    pub fn c_name(&self) -> &'static str {
        match self {
            FieldType::U32 => "uint32_t",
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

    fn event(&mut self, name: &'static str, id: usize) -> EventBuilder {
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
        self.event.size += ty.size();
        self
    }
}

pub fn all() -> Vec<Event> {
    use FieldType::*;

    let mut builder = EventsBuilder::new();

    builder
        .event("mouse_down", 1)
        .field("x", U32)
        .field("y", U32)
        .field("button", U32);

    builder
        .event("mouse_up", 2)
        .field("x", U32)
        .field("y", U32)
        .field("button", U32);

    builder
        .event("mouse_motion", 3)
        .field("x", U32)
        .field("y", U32);

    builder.event("quit", 4);

    builder.events()
}
