#![cfg_attr(test, feature(test))]

use wasmi::{
    Caller as WasmiCaller, Engine as WasmiEngine, Linker as WasmiLinker, Module as WasmiModule,
    Store as WasmiStore,
};
use wasmtime::{
    Caller as WasmtimeCaller, Engine as WasmtimeEngine, Linker as WasmtimeLinker,
    Module as WasmtimeModule, Store as WasmtimeStore,
};

pub const HOST_CALLS_PER_INVOCATION: u32 = 1000;

pub fn module_wat() -> String {
    format!(
        r#"
        (module
            (import "host" "hello" (func $host_hello (param i32)))
            (func $hello_loop (local $remaining i32)
                (local.set $remaining (i32.const {HOST_CALLS_PER_INVOCATION}))
                (loop $repeat
                    (call $host_hello (i32.const 3))
                    (local.set $remaining (i32.sub (local.get $remaining) (i32.const 1)))
                    (br_if $repeat (i32.gt_s (local.get $remaining) (i32.const 0)))
                )
            )
            (func (export "hello")
                (call $hello_loop)
            )
        )
    "#
    )
}

pub fn module_wasm() -> Vec<u8> {
    wat::parse_str(module_wat()).expect("WAT module should be valid")
}

pub fn call_hello_1000_times_wasmi() -> Result<u32, wasmi::Error> {
    let wasm = module_wasm();
    let engine = WasmiEngine::default();
    let module = WasmiModule::new(&engine, &wasm)?;
    let mut store = WasmiStore::new(&engine, 0_u32);
    let mut linker = WasmiLinker::new(&engine);

    linker.func_wrap(
        "host",
        "hello",
        |mut caller: WasmiCaller<'_, u32>, _param: i32| {
            *caller.data_mut() += 1;
        },
    )?;

    let instance = linker.instantiate_and_start(&mut store, &module)?;
    let hello = instance.get_typed_func::<(), ()>(&store, "hello")?;
    hello.call(&mut store, ())?;
    Ok(*store.data())
}

pub fn call_hello_1000_times_wasmtime() -> Result<u32, wasmtime::Error> {
    let wasm = module_wasm();
    let engine = WasmtimeEngine::default();
    let module = WasmtimeModule::new(&engine, &wasm)?;
    let mut store = WasmtimeStore::new(&engine, 0_u32);
    let mut linker = WasmtimeLinker::new(&engine);

    linker.func_wrap(
        "host",
        "hello",
        |mut caller: WasmtimeCaller<'_, u32>, _param: i32| {
            *caller.data_mut() += 1;
        },
    )?;

    let instance = linker.instantiate(&mut store, &module)?;
    let hello = instance.get_typed_func::<(), ()>(&mut store, "hello")?;
    hello.call(&mut store, ())?;
    Ok(*store.data())
}

#[cfg(test)]
mod benches {
    extern crate test;

    use super::{
        HOST_CALLS_PER_INVOCATION, call_hello_1000_times_wasmi, call_hello_1000_times_wasmtime,
        module_wasm,
    };
    use test::{Bencher, black_box};
    use wasmi::{
        Caller as WasmiCaller, Engine as WasmiEngine, Instance as WasmiInstance,
        Linker as WasmiLinker, Module as WasmiModule, Store as WasmiStore,
        TypedFunc as WasmiTypedFunc,
    };
    use wasmtime::{
        Caller as WasmtimeCaller, Engine as WasmtimeEngine, Instance as WasmtimeInstance,
        Linker as WasmtimeLinker, Module as WasmtimeModule, Store as WasmtimeStore,
        TypedFunc as WasmtimeTypedFunc,
    };

    fn instantiate_wasmi_benchmark_module()
    -> (WasmiStore<u32>, WasmiInstance, WasmiTypedFunc<(), ()>) {
        let wasm = module_wasm();
        let engine = WasmiEngine::default();
        let module = WasmiModule::new(&engine, &wasm).expect("module compilation should succeed");
        let mut store = WasmiStore::new(&engine, 0_u32);
        let mut linker = WasmiLinker::new(&engine);

        linker
            .func_wrap(
                "host",
                "hello",
                |mut caller: WasmiCaller<'_, u32>, _param: i32| {
                    *caller.data_mut() += 1;
                },
            )
            .expect("host function definition should succeed");

        let instance = linker
            .instantiate_and_start(&mut store, &module)
            .expect("instantiation should succeed");
        let hello = instance
            .get_typed_func::<(), ()>(&store, "hello")
            .expect("exported function should exist");

        (store, instance, hello)
    }

    fn instantiate_wasmtime_benchmark_module() -> (
        WasmtimeStore<u32>,
        WasmtimeInstance,
        WasmtimeTypedFunc<(), ()>,
    ) {
        let wasm = module_wasm();
        let engine = WasmtimeEngine::default();
        let module =
            WasmtimeModule::new(&engine, &wasm).expect("module compilation should succeed");
        let mut store = WasmtimeStore::new(&engine, 0_u32);
        let mut linker = WasmtimeLinker::new(&engine);

        linker
            .func_wrap(
                "host",
                "hello",
                |mut caller: WasmtimeCaller<'_, u32>, _param: i32| {
                    *caller.data_mut() += 1;
                },
            )
            .expect("host function definition should succeed");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("instantiation should succeed");
        let hello = instance
            .get_typed_func::<(), ()>(&mut store, "hello")
            .expect("exported function should exist");

        (store, instance, hello)
    }

    #[bench]
    fn bench_host_hello_1000x_wasmi(b: &mut Bencher) {
        let (mut store, _instance, hello) = instantiate_wasmi_benchmark_module();

        b.iter(|| {
            hello
                .call(&mut store, ())
                .expect("benchmark execution should succeed");
            black_box(*store.data());
        });
    }

    #[bench]
    fn bench_host_hello_1000x_wasmtime(b: &mut Bencher) {
        let (mut store, _instance, hello) = instantiate_wasmtime_benchmark_module();

        b.iter(|| {
            hello
                .call(&mut store, ())
                .expect("benchmark execution should succeed");
            black_box(*store.data());
        });
    }
}
