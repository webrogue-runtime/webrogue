use std::collections::BTreeSet;

use crate::{Enum, EnumCase, Event, Field, FieldType, RawType};

pub struct EventsBuilder {
    events: Vec<Event>,
    enums: Vec<Enum>,
}

impl EventsBuilder {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            enums: Vec::new(),
        }
    }

    pub fn add_event<'a>(&'a mut self, name: &'static str, id: usize) -> EventBuilder<'a> {
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

    pub fn events(self) -> Vec<Event> {
        self.events
    }

    pub fn add_enum<'a>(&'a mut self, name: &'static str, ty: RawType) -> EnumBuilder<'a> {
        self.enums.push(Enum {
            name,
            ty,
            cases: Vec::new(),
        });
        EnumBuilder {
            values: BTreeSet::new(),
            r#enum: self.enums.last_mut().unwrap(),
        }
    }

    pub fn enums(self) -> Vec<Enum> {
        self.enums
    }
}

pub struct EventBuilder<'a> {
    event: &'a mut Event,
}

impl EventBuilder<'_> {
    pub fn field(&mut self, name: &'static str, ty: RawType) -> &mut Self {
        self.event.fields.push(Field {
            name,
            offset: self.event.size,
            ty: FieldType::Raw(ty),
        });
        self.align();
        self
    }

    pub fn enum_field(&mut self, name: &'static str, ty: Enum) -> &mut Self {
        self.event.fields.push(Field {
            name,
            offset: self.event.size,
            ty: FieldType::Enum(ty),
        });
        self.align();
        self
    }

    pub fn bytes_field(&mut self, name: &'static str, len: usize) -> &mut Self {
        self.event.fields.push(Field {
            name,
            offset: self.event.size,
            ty: FieldType::Bytes(len),
        });
        self.align();
        self
    }

    pub fn align(&mut self) {
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

pub struct EnumBuilder<'a> {
    values: BTreeSet<u64>,
    r#enum: &'a mut Enum,
}

impl EnumBuilder<'_> {
    pub fn add_case(&mut self, name: &'static str, value: u64) -> &mut Self {
        if !self.values.insert(value) {
            panic!(
                "duplicate case \"{}\" = {} in enum \"{}\"",
                name, value, self.r#enum.name
            )
        }
        self.r#enum.cases.push(EnumCase { name, value });
        self
    }

    pub fn build(&self) -> Enum {
        self.r#enum.clone()
    }
}
