fn main() {
    let wasmi_calls = host_fn_benchmark::call_hello_million_times_times_wasmi()
        .expect("wasmi module should call the host function successfully");
    let wasmtime_calls = host_fn_benchmark::call_hello_million_times_times_wasmtime()
        .expect("wasmtime module should call the host function successfully");
    let wasmer_calls = host_fn_benchmark::call_hello_million_times_times_wasmer()
        .expect("wasmer module should call the host function successfully");
    println!("Executed {wasmi_calls} host calls with wasmi");
    println!("Executed {wasmtime_calls} host calls with wasmtime");
    println!("Executed {wasmer_calls} host calls with wasmer");
}
