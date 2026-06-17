use std::fmt::Debug;
use std::ops::Range;

use rangemap::RangeMap;
use tracing::debug;

#[derive(Clone, Debug, PartialEq)]
pub struct Destination<K> {
    pub(crate) bus: u8,
    pub(crate) device: u8,
    pub(crate) function: u8,
    pub(crate) bar: u8,
    pub(crate) base: K,
}

#[derive(Default)]
pub struct Router<K> {
    map: RangeMap<K, Destination<K>>,
}

impl<K> Router<K>
where
    K: Clone + Debug + Ord,
{
    pub fn register_handler(
        &mut self,
        range: Range<K>,
        bus: u8,
        device: u8,
        function: u8,
        bar: u8,
    ) {
        debug!(bus, device, function, bar, ?range, "update handler");

        self.map.insert(
            range.clone(),
            Destination {
                bus,
                device,
                function,
                bar,
                base: range.start,
            },
        );
    }

    pub fn unregister_handler(&mut self, bus: u8, device: u8, function: u8, bar: u8) {
        debug!(bus, device, function, bar, "update handler");

        let mut to_remove = vec![];

        for (range, dest) in self.map.iter() {
            if dest.bus == bus
                && dest.device == device
                && dest.function == function
                && dest.bar == bar
            {
                to_remove.push(range.clone());
            }
        }

        for range in to_remove {
            self.map.remove(range);
        }
    }

    pub fn get_handler(&self, addr: K) -> Option<Destination<K>> {
        self.map.get(&addr).cloned()
    }
}
