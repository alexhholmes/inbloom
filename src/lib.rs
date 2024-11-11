use std::hash::{DefaultHasher, Hash, Hasher};
use std::cmp;

#[derive(Copy, Clone, Debug)]
enum Cardinality {
    Evaluated(f64),
    Expired,
}

#[derive(Clone, Debug)]
pub struct HyperLogLog<H: Hasher, const R: u8> {
    hasher: H,
    registers: Vec<u8>,
    cardinality: Cardinality,
}

impl<H: Hasher, const R: u8> HyperLogLog<H, R> {
    pub fn new_with_hasher(hasher: H) -> Self {
        Self {
            hasher,
            registers: vec![0; 1 << R],
            cardinality: Cardinality::Expired,
        }
    }

    pub fn insert<V: Hash>(&mut self, elem: V) {
        elem.hash(&mut self.hasher);
        let hash = self.hasher.finish();
        let register = ((1 << R & hash) >> 56) as usize;
        self.registers[register] = cmp::max(self.registers[register], hash.leading_zeros() as u8);
        self.cardinality = Cardinality::Expired;
    }

    pub fn merge(&mut self, other: &Self) {
        for (idx, reg) in other.registers.iter().enumerate() {
            self.registers[idx] = cmp::max(self.registers[idx], *reg)
        }
    }

    pub fn evaluate(&mut self) -> f64 {
        match self.cardinality {
            Cardinality::Evaluated(eval) => eval,
            Cardinality::Expired => {
                let sum: f64 = self.registers
                    .iter()
                    .map(|&x| x as f64)
                    .sum();
                let avg = sum / (1 << R) as f64;
                self.cardinality = Cardinality::Evaluated(avg);
                avg
            }
        }
    }
}

impl<const R: u8> HyperLogLog<DefaultHasher, R> {
    pub fn new() -> Self {
        Self::new_with_hasher(DefaultHasher::new())
    }
}

impl Default for HyperLogLog<DefaultHasher, 8> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut hll = HyperLogLog::<_, 8>::new();
        hll.insert("test1".as_bytes());
        println!("{}", hll.evaluate())
    }
}
