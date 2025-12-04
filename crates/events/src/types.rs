#[derive(Clone)]
pub struct Event {
    pub name: &'static str,
    pub id: usize,
    pub fields: Vec<Field>,
    pub size: usize,
}

impl Event {
    pub fn c_case_name(&self) -> String {
        format!(
            "WEBROGUE_EVENT_TYPE_{}",
            self.name.replace(' ', "_").to_uppercase(),
        )
    }

    pub fn c_struct_name(&self) -> String {
        format!("webrogue_event_{}", self.name.replace(' ', "_"),)
    }

    pub fn c_union_name(&self) -> String {
        self.name.replace(' ', "_")
    }

    pub fn rust_name(&self) -> String {
        self.name.replace(' ', "_")
    }
}

#[derive(Clone)]
pub struct Enum {
    pub name: &'static str,
    pub ty: RawType,
    pub cases: Vec<EnumCase>,
}

impl Enum {
    pub fn c_name(&self) -> String {
        format!("webrogue_{}", self.name.replace(' ', "_"))
    }

    pub fn rust_name(&self) -> String {
        self.name
            .split(' ')
            .map(|word| {
                let mut word = word.to_owned();
                let first_char = word.remove(0).to_ascii_uppercase();
                word.insert(0, first_char);
                word
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

#[derive(Clone)]
pub struct EnumCase {
    pub name: &'static str,
    pub value: u64,
}

impl EnumCase {
    pub fn c_name(&self, r#enum: &Enum) -> String {
        format!("{}_{}", r#enum.c_name(), self.name.replace(' ', "_")).to_uppercase()
    }

    pub fn rust_name(&self) -> String {
        self.name
            .split(' ')
            .map(|word| {
                let mut word = word.to_owned();
                let first_char = word.remove(0).to_ascii_uppercase();
                word.insert(0, first_char);
                word
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

#[derive(Clone)]
pub struct Field {
    pub name: &'static str,
    pub offset: usize,
    pub ty: FieldType,
}

impl Field {
    pub fn c_name(&self) -> String {
        self.name.replace(' ', "_")
    }

    pub fn rust_name(&self) -> String {
        self.name.replace(' ', "_")
    }
}
#[derive(Clone)]
pub enum FieldType {
    Enum(Enum),
    Raw(RawType),
    Bytes(usize),
}

impl FieldType {
    pub fn c_name(&self) -> String {
        match self {
            FieldType::Enum(r#enum) => r#enum.c_name(),
            FieldType::Raw(raw_type) => raw_type.c_name().to_owned(),
            FieldType::Bytes(_) => "uint8_t".to_owned(),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            FieldType::Enum(r#enum) => r#enum.ty.size(),
            FieldType::Raw(raw_type) => raw_type.size(),
            FieldType::Bytes(len) => *len,
        }
    }
}

#[derive(Clone, Copy)]
pub enum RawType {
    U32,
    U16,
    Bool,
    U8,
}

impl RawType {
    pub fn size(&self) -> usize {
        match self {
            RawType::U32 => 4,
            RawType::U16 => 2,
            RawType::Bool => 1,
            RawType::U8 => 1,
        }
    }

    pub fn c_name(&self) -> &'static str {
        match self {
            RawType::U32 => "uint32_t",
            RawType::U16 => "uint16_t",
            RawType::Bool => "uint8_t",
            RawType::U8 => "uint8_t",
        }
    }

    pub fn c_max(&self) -> String {
        format!("0x{}", "FF".repeat(self.size()))
    }
}
