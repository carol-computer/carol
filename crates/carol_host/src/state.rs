#![allow(clippy::type_complexity)]
use crate::{BinaryId, CompiledBinary, Executor, MachineId};
use carol_bls as bls;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct State {
    pub bls_keypair: bls::KeyPair,
    pub exec: ExecutorState,
}

impl State {
    pub fn new(bls_keypair: bls::KeyPair) -> Self {
        Self {
            bls_keypair,
            exec: ExecutorState::default(),
        }
    }
}

#[derive(Clone, Default)]
pub struct ExecutorState {
    executor: Executor,
    binaries: Arc<Mutex<HashMap<BinaryId, Arc<CompiledBinary>>>>,
    machines: Arc<Mutex<HashMap<MachineId, (BinaryId, Arc<Vec<u8>>)>>>,
}

impl ExecutorState {
    pub fn new(executor: Executor) -> Self {
        Self {
            executor,
            binaries: Default::default(),
            machines: Default::default(),
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
}
