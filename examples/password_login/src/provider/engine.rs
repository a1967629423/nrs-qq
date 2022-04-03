use nrs_qq::engine::{CoreCryptoRng, TimerProvider};

// engine provider
pub struct MyTimeProvider;
impl TimerProvider for MyTimeProvider {
    fn now_timestamp_nanos(&self) -> i64 {
        chrono::Utc::now().timestamp_nanos()
    }
}

pub struct MyRandomProvidr;
impl rand::RngCore for MyRandomProvidr {
    fn next_u32(&mut self) -> u32 {
        rand::thread_rng().next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        rand::thread_rng().next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        rand::thread_rng().fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        rand::thread_rng().try_fill_bytes(dest)
    }
}
impl rand::CryptoRng for MyRandomProvidr {}
impl CoreCryptoRng for MyRandomProvidr {}
