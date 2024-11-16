use std::cmp;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::marker::PhantomData;

#[derive(Copy, Clone, Debug)]
enum Cardinality {
    Evaluated(f64),
    Expired,
}

#[derive(Clone, Debug)]
pub struct HyperLogLog<H: Hasher + Default, const R: u8> {
    registers: Vec<u8>,
    cardinality: Cardinality,
    hasher: PhantomData<H>,
}

impl<H: Hasher + Default, const R: u8> HyperLogLog<H, R> {
    const ASSERT_VALID_R: () = assert!(R > 3 && R < 17);

    pub fn with_hasher() -> Self {
        let _ = Self::ASSERT_VALID_R;
        Self {
            registers: vec![0; 1 << R],
            cardinality: Cardinality::Expired,
            hasher: Default::default(),
        }
    }

    pub fn insert<V: Hash>(&mut self, elem: V) {
        let mut hasher = H::default();
        elem.hash(&mut hasher);
        let hash = hasher.finish();
        let register = (hash >> (64 - R)) as usize;
        self.registers[register] = cmp::max(self.registers[register], (hash << R).leading_zeros() as u8);
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

    pub fn reset(&mut self) {
        self.registers.fill(0);
        self.cardinality = Cardinality::Expired;
    }
}

impl<const R: u8> HyperLogLog<DefaultHasher, R> {
    fn new() -> Self {
        HyperLogLog::<DefaultHasher, R>::with_hasher()
    }
}

impl Default for HyperLogLog<DefaultHasher, 8> {
    fn default() -> Self {
        Self::with_hasher()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut hll = HyperLogLog::default();
        hll.insert("test1".as_bytes());
        let mut hll = HyperLogLog::<_, 8>::new();
        hll.insert("test1".as_bytes());
    }
}
