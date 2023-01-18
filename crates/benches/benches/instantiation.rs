use criterion::{criterion_group, criterion_main, Criterion};
use wasmtime::*;
use wasmtime_wasi::{sync, WasiCtx, WasiCtxBuilder};

const RUVY: &'static [u8] = include_bytes!("ruvy.wizened.wasm");

struct VM {
    pub engine: Engine,
    config: Config,
}

struct Ctx {
    pub wasi: WasiCtx,
}

impl Default for Ctx {
    fn default() -> Self {
        Self {
            wasi: WasiCtxBuilder::new().inherit_stdio().build(),
        }
    }
}

impl Default for VM {
    fn default() -> Self {
        let config = Config::new();
        let engine = Engine::new(&config).unwrap();

        Self { config, engine }
    }
}

impl VM {
    pub fn with_pooling_strategy(strategy: InstanceAllocationStrategy) -> Self {
        let mut config = Config::new();
        config.allocation_strategy(strategy);
        let engine = Engine::new(&config).unwrap();
        Self { engine, config }
    }
}

fn instantiate(c: &mut Criterion) {
    c.bench_function("with on demand allocation", |b| {
        let vm = VM::default();
        let context = Ctx::default();
        let mut store = Store::new(&vm.engine, context);
        let mut linker = Linker::<Ctx>::new(&vm.engine);
        sync::add_to_linker(&mut linker, |c: &mut Ctx| &mut c.wasi).unwrap();
        let module = Module::new(&vm.engine, RUVY).unwrap();
        b.iter(|| {
            linker.instantiate(&mut store, &module).unwrap();
        })
    });

    c.bench_function("with pooling: reuse affinity", |b| {
        let mut cfg = PoolingAllocationConfig::default();
        // memory 0 has 947 pages.
        cfg.instance_memory_pages(1000);
        cfg.instance_count(10000);
        let vm = VM::with_pooling_strategy(InstanceAllocationStrategy::Pooling(cfg));
        let context = Ctx::default();
        let mut store = Store::new(&vm.engine, context);
        let mut linker = Linker::<Ctx>::new(&vm.engine);
        sync::add_to_linker(&mut linker, |c: &mut Ctx| &mut c.wasi).unwrap();
        let module = Module::new(&vm.engine, RUVY).unwrap();
        b.iter(|| {
            linker.instantiate(&mut store, &module).unwrap();
        })
    });

    c.bench_function("with pooling: next available", |b| {
        let mut cfg = PoolingAllocationConfig::default();
        // memory 0 has 947 pages.
        cfg.instance_memory_pages(1000);
        cfg.instance_count(10000);
        cfg.strategy(PoolingAllocationStrategy::NextAvailable);
        let vm = VM::with_pooling_strategy(InstanceAllocationStrategy::Pooling(cfg));
        let context = Ctx::default();
        let mut store = Store::new(&vm.engine, context);
        let mut linker = Linker::<Ctx>::new(&vm.engine);
        sync::add_to_linker(&mut linker, |c: &mut Ctx| &mut c.wasi).unwrap();
        let module = Module::new(&vm.engine, RUVY).unwrap();
        b.iter(|| {
            linker.instantiate(&mut store, &module).unwrap();
        })
    });
}

criterion_group!(benches, instantiate);
criterion_main!(benches);
