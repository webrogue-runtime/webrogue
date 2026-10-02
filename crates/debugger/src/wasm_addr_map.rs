use std::collections::BTreeMap;

pub struct WasmAddrMap<Value: Clone + Ord> {
    map: BTreeMap<Value, u32>, // <(debug_index_in_store, is_shared), module_id_for_wasm_addr>
}

pub type MemoryAddrMap = WasmAddrMap<(u64, bool)>;
pub type ModuleAddrMap = WasmAddrMap<u64>;

impl<Value: Clone + Ord> WasmAddrMap<Value> {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn get_idx(&mut self, val: Value) -> u32 {
        if let Some(idx) = self.map.get(&val) {
            return *idx;
        }
        let idx = self.map.len() as u32;
        self.map.insert(val.clone(), idx);
        idx
    }
}
