use tokio::sync::Mutex;
async fn f(m: &Mutex<i32>) { let guard = m.lock().await; work().await; }
