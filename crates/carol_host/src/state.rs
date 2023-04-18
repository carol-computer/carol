use crate::{BinaryId, CompiledBinary, Executor, MachineId};
use carol_bls as bls;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct State {
    executor: Arc<Executor>,
    binaries: Arc<Mutex<HashMap<BinaryId, Arc<CompiledBinary>>>>,
    machines: Arc<Mutex<HashMap<MachineId, (BinaryId, Arc<Vec<u8>>)>>>,
    bls_keypair: bls::KeyPair,
}

impl State {
    pub fn new(executor: Executor, bls_keypair: bls::KeyPair) -> Self {
        Self {
            executor: Arc::new(executor),
            binaries: Default::default(),
            machines: Default::default(),
            bls_keypair,
        }
    }

    pub fn get_binary(&self, binary_id: BinaryId) -> Option<Arc<CompiledBinary>> {
        self.binaries.lock().unwrap().get(&binary_id).cloned()
    }

    pub fn insert_binary(&self, compiled_binary: CompiledBinary) {
        self.binaries
            .lock()
            .unwrap()
            .insert(compiled_binary.binary_id, Arc::new(compiled_binary));
    }

    pub fn get_machine(&self, machine_id: MachineId) -> Option<(BinaryId, Arc<Vec<u8>>)> {
        self.machines.lock().unwrap().get(&machine_id).cloned()
    }

    pub fn insert_machine(&self, binary_id: BinaryId, params: Vec<u8>) -> (bool, MachineId) {
        let machine_id = MachineId::new(binary_id, &params);
        let already_existed = self
            .machines
            .lock()
            .unwrap()
            .insert(machine_id, (binary_id, Arc::new(params)))
            .is_some();
        (already_existed, machine_id)
    }

    pub fn executor(&self) -> &Executor {
        &self.executor
    }

    pub fn bls_keypair(&self) -> &bls::KeyPair {
        &self.bls_keypair
    }
}
