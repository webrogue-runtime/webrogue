
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

#[derive(Clone)]
pub enum FieldType {
    U32,
}
impl FieldType {
    pub fn size(&self) -> usize {
        match self {
            FieldType::U32 => 4
        }
    }

    pub fn c_name(&self) -> &'static str {
        match self {
            FieldType::U32 => "uint32_t"
        }
    }
}

pub fn all() -> Vec<Event> {
    let mut events = Vec::new();
    let mut current_fields = Vec::new();

    let mut offset = 4;
    macro_rules! field {
        ($name:expr, $ty:expr) => {
            current_fields.push(Field {
                name: $name,
                offset: offset,
                ty: $ty
            });
            offset += $ty.size();
        };
    }

    macro_rules! event {
        ($name:expr, $id:expr) => {
            events.push(Event {
                name: $name,
                id: $id,
                fields: current_fields.clone(),
                size: offset
            });
            current_fields.clear();
            offset = 4;
        };
    }
    
    field!("x", FieldType::U32);
    field!("y", FieldType::U32);
    field!("button", FieldType::U32);
    event!("mouse_down", 1);

    field!("x", FieldType::U32);
    field!("y", FieldType::U32);
    field!("button", FieldType::U32);
    event!("mouse_up", 2);

    field!("x", FieldType::U32);
    field!("y", FieldType::U32);
    event!("mouse_motion", 3);

    event!("quit", 4);

    let _ = offset; // to silence a warning
    events
}
