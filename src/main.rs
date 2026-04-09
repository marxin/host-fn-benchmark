fn main() {
    let wasmi_calls = host_fn_benchmark::call_hello_1000_times_wasmi()
        .expect("wasmi module should call the host function successfully");
    let wasmtime_calls = host_fn_benchmark::call_hello_1000_times_wasmtime()
        .expect("wasmtime module should call the host function successfully");
    let wasmer_calls = host_fn_benchmark::call_hello_1000_times_wasmer()
        .expect("wasmer module should call the host function successfully");
    let wasmedge_calls = host_fn_benchmark::call_hello_1000_times_wasmedge()
        .expect("wasmedge module should call the host function successfully");
    println!("Executed {wasmi_calls} host calls with wasmi");
    println!("Executed {wasmtime_calls} host calls with wasmtime");
    println!("Executed {wasmer_calls} host calls with wasmer");
    println!("Executed {wasmedge_calls} host calls with wasmedge");
}
