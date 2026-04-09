fn main() {
    let calls = host_fn_benchmark::call_hello_1000_times()
        .expect("Wasm module should call the host function successfully");
    println!("Executed {calls} host calls");
}
