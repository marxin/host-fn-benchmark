#![cfg_attr(test, feature(test))]

use wasmi::{Caller, Engine, Linker, Module, Store};

const HOST_CALLS_PER_INVOCATION: u32 = 1000;

fn module_wat() -> String {
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

pub fn call_hello_1000_times() -> Result<u32, wasmi::Error> {
    let wasm = wat::parse_str(module_wat()).expect("WAT module should be valid");
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm)?;

    type HostState = u32;
    let mut store = Store::new(&engine, 0);
    let mut linker = <Linker<HostState>>::new(&engine);

    linker.func_wrap(
        "host",
        "hello",
        |_caller: Caller<'_, HostState>, param: i32| -> i32 { param + 1 },
    )?;

    let instance = linker.instantiate_and_start(&mut store, &module)?;
    instance
        .get_typed_func::<(), ()>(&store, "hello")?
        .call(&mut store, ())?;

    Ok(*store.data())
}

#[cfg(test)]
mod benches {
    extern crate test;

    use super::module_wat;
    use test::{Bencher, black_box};
    use wasmi::{Caller, Engine, Instance, Linker, Module, Store, TypedFunc};

    fn instantiate_benchmark_module() -> (Store<u32>, Instance, TypedFunc<(), ()>) {
        let wasm = wat::parse_str(module_wat()).expect("WAT module should be valid");
        let engine = Engine::default();
        let module = Module::new(&engine, &wasm).expect("module compilation should succeed");
        let mut store = Store::new(&engine, 0_u32);
        let mut linker = Linker::new(&engine);

        linker
            .func_wrap(
                "host",
                "hello",
                |mut caller: Caller<'_, u32>, _param: i32| {
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

    #[bench]
    fn bench_host_hello_1000x(b: &mut Bencher) {
        let (mut store, _instance, hello) = instantiate_benchmark_module();

        b.iter(|| {
            hello
                .call(&mut store, ())
                .expect("benchmark execution should succeed");
            black_box(*store.data());
        });
    }
}
