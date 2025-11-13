use alloc::collections::{BTreeMap, VecDeque};
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

pub struct Message {
    pub sender: u64,
    pub data: Vec<u8>,
}
struct Endpoint {
    name: Option<String>,
    messages: VecDeque<Message>,
    waiting_threads: Vec<u64>,
}

impl Endpoint {
    const fn new() -> Self {
        Self {
            name: None,
            messages: VecDeque::new(),
            waiting_threads: Vec::new(),
        }
    }
}
static NEXT_PORT_ID: AtomicU64 = AtomicU64::new(1);
static ENDPOINTS: Mutex<BTreeMap<u64, Endpoint>> = Mutex::new(BTreeMap::new());
static NAME_TO_PORT: Mutex<BTreeMap<String, u64>> = Mutex::new(BTreeMap::new());

pub fn init() {
    // Пустая инициализация — статические структуры уже инициализированы
}

pub fn create_endpoint(name: Option<&str>) -> u64 {
    let port_id = NEXT_PORT_ID.fetch_add(1, Ordering::Relaxed);
    let mut endpoints = ENDPOINTS.lock();
    endpoints.insert(port_id, Endpoint::new());

    if let Some(name) = name {
        let name = name.to_string();
        endpoints.get_mut(&port_id).unwrap().name = Some(name.clone());
        NAME_TO_PORT.lock().insert(name, port_id);
    }

    port_id
}

pub fn resolve_port(name: &str) -> Option<u64> {
    NAME_TO_PORT.lock().get(name).copied()
}

pub fn send(recipient: u64, data: &[u8]) -> Result<(), &'static str> {
    let mut endpoints = ENDPOINTS.lock();
    if let Some(endpoint) = endpoints.get_mut(&recipient) {
        let msg = Message {
            sender: 0, // можно расширить до реального sender_id
            data: data.to_vec(),
        };
        endpoint.messages.push_back(msg);
        Ok(())
    } else {
        Err("Endpoint not found")
    }
}

pub fn receive(port: u64) -> Option<Message> {
    let mut endpoints = ENDPOINTS.lock();
    if let Some(endpoint) = endpoints.get_mut(&port) {
        endpoint.messages.pop_front()
    } else {
        None
    }
}

pub fn receive_blocking(port: u64) -> Message {
    loop {
        if let Some(msg) = receive(port) {
            return msg;
        }
        // yield CPU to scheduler
        crate::scheduler::yield_now();
    }
}

pub fn close_endpoint(port: u64) {
    let mut endpoints = ENDPOINTS.lock();
    if let Some(endpoint) = endpoints.remove(&port) {
        if let Some(name) = endpoint.name {
            NAME_TO_PORT.lock().remove(&name);
        }
    }
}