use std::cell::Cell;

const POOL_SIZE: u32 = 3;
const REQUESTS: u32 = 5;

struct Pool {
    available: Cell<u32>,
}

struct Connection<'a> {
    pool: &'a Pool,
}

impl Pool {
    fn checkout(&self) -> Option<Connection<'_>> {
        let n = self.available.get();
        if n == 0 {
            return None;
        }
        self.available.set(n - 1);
        Some(Connection { pool: self })
    }
}

impl Drop for Connection<'_> {
    fn drop(&mut self) {
        self.pool.available.set(self.pool.available.get() + 1);
    }
}

fn handle_request(pool: &Pool) -> bool {
    match pool.checkout() {
        Some(conn) => {
            std::mem::forget(conn);
            true
        }
        None => false,
    }
}

fn main() {
    let pool = Pool { available: Cell::new(POOL_SIZE) };
    let served = (0..REQUESTS).filter(|_| handle_request(&pool)).count();
    println!("served {} of {} requests, connections available: {}", served, REQUESTS, pool.available.get());
}
