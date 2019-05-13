use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::collections::HashSet;
use std::sync::{Arc, Condvar, Mutex};

#[derive(Copy, Clone, PartialEq, Eq, Hash, FromPrimitive, ToPrimitive, Debug)]
pub enum Key {
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
}

impl Key {
    pub fn from_num(num: u8) -> Option<Key> {
        FromPrimitive::from_u8(num)
    }

    pub fn to_num(&self) -> u8 {
        ToPrimitive::to_u8(self).unwrap()
    }
}

#[derive(Clone)]
pub struct Keyboard {
    keys: Arc<(Mutex<HashSet<Key>>, Condvar)>,
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard {
            keys: Arc::new((Mutex::new(HashSet::new()), Condvar::new())),
        }
    }

    pub fn is_pressed(&self, key: &Key) -> bool {
        let k = self.keys.0.lock().unwrap();
        (*k).contains(key)
    }

    pub fn set_pressed(&self, key: Key) {
        let mut k = self.keys.0.lock().unwrap();
        (*k).insert(key);
        self.keys.1.notify_all();
    }

    pub fn set_unpressed(&self, key: Key) {
        let mut k = self.keys.0.lock().unwrap();
        (*k).remove(&key);
        self.keys.1.notify_all();
    }

    pub fn wait(&mut self) -> Key {
        let guard = self.keys.0.lock().unwrap();
        let current_keys = (*guard).clone();

        let new_keys = self.keys.1.wait_until(
            guard,
            |x| {
                let result: HashSet<&Key> = x.difference(&current_keys).collect();
                !result.is_empty()
            }
        ).unwrap();

        let result: HashSet<&Key> = (*new_keys).difference(&current_keys).collect();
        **result.iter().nth(0).unwrap()
    }
}
