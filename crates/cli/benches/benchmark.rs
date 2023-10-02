use std::{
    ffi::OsStr,
    fmt::{self, Display},
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    str,
};

use anyhow::{bail, Result};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use wasi_common::{
    pipe::{ReadPipe, WritePipe},
    WasiCtx,
};
use wasmtime::{Engine, Linker, Module, Store, TypedFunc};
use wasmtime_wasi::WasiCtxBuilder;

mod helpers;

pub fn criterion_benchmark(c: &mut Criterion) {
    let engine = Engine::default();
    let cases = vec![
        WasmCase::new(
            CompilationStrategy::WasiVFSRubyWasm,
            "benches/scripts/hello_world/hello_world.rb".into(),
        )
        .unwrap(),
        WasmCase::new(
            CompilationStrategy::Ruvy(None),
            "benches/scripts/hello_world/hello_world.rb".into(),
        )
        .unwrap(),
        WasmCase::new(
            CompilationStrategy::WasiVFSRubyWasm,
            "benches/scripts/transformer/ruby_wasm_entry.rb".into(),
        )
        .unwrap(),
        WasmCase::new(
            CompilationStrategy::Ruvy(Some("benches/scripts/transformer/preload".into())),
            "benches/scripts/transformer/ruvy_entry.rb".into(),
        )
        .unwrap(),
    ];
    for case in cases {
        c.bench_with_input(BenchmarkId::new("compile", &case), &case, |b, script| {
            b.iter(|| Module::new(&engine, &script.wasm).unwrap())
        });

        c.bench_with_input(BenchmarkId::new("run", &case), &case, |b, script| {
            b.iter_with_setup(
                || script.setup_for_run(&engine).unwrap(),
                |(start_func, mut store)| start_func.call(&mut store, ()).unwrap(),
            )
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

struct WasmCase {
    name: String,
    wasm: Vec<u8>,
    wasi_args: Vec<String>,
    input: Vec<u8>,
}

impl Display for WasmCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl WasmCase {
    fn new(strategy: CompilationStrategy, entrypoint: Entrypoint) -> Result<WasmCase> {
        let name = format!("{}-{}", strategy, entrypoint.parent_dirname);
        let output_path = helpers::cargo_target_tmpdir()
            .join(&name)
            .with_extension("wasm");
        let exit_status = strategy.compile_wasm(&output_path, &entrypoint)?;
        if !exit_status.success() {
            bail!("Failed to build Wasm module");
        }

        let input_file = &entrypoint.input_path;
        let input = if input_file.exists() {
            fs::read(input_file)?
        } else {
            vec![]
        };

        Ok(Self {
            name,
            wasm: fs::read(output_path)?,
            wasi_args: strategy.wasi_args(&entrypoint),
            input,
        })
    }

    fn setup_for_run(&self, engine: &Engine) -> Result<(TypedFunc<(), ()>, Store<WasiCtx>)> {
        let mut linker = Linker::new(engine);
        let wasi = WasiCtxBuilder::new()
            .stdin(Box::new(ReadPipe::from(&self.input[..])))
            .stdout(Box::new(WritePipe::new_in_memory()))
            .stderr(Box::new(WritePipe::new_in_memory()))
            .args(&self.wasi_args)?
            .build();
        wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();
        let mut store = Store::new(engine, wasi);
        let module = Module::new(engine, &self.wasm)?;
        let instance = linker.instantiate(&mut store, &module)?;
        let func = instance.get_typed_func(&mut store, "_start")?;
        Ok((func, store))
    }
}

enum CompilationStrategy {
    WasiVFSRubyWasm,
    Ruvy(Option<PathBuf>),
}

impl CompilationStrategy {
    fn compile_wasm(&self, output_path: &Path, entrypoint: &Entrypoint) -> Result<ExitStatus> {
        match self {
            CompilationStrategy::WasiVFSRubyWasm => {
                let ruby_wasm = helpers::ruby_wasm()?;
                let wasi_vfs = helpers::wasi_vfs()?;
                Ok(Command::new(wasi_vfs)
                    .arg("pack")
                    .arg(&ruby_wasm)
                    .arg("--mapdir")
                    .arg(format!("/src::{}", entrypoint.parent_path))
                    // Examples online show mapping the `/usr` directory, however this
                    // breaks when using a `minimal` instead of `full` profile of
                    // ruby.wasm, so we don't map that directory here.
                    .arg("-o")
                    .arg(output_path.as_os_str())
                    .status()?)
            }
            CompilationStrategy::Ruvy(preload) => {
                let ruvy = env!("CARGO_BIN_EXE_ruvy");
                let mut args = vec![entrypoint.path, OsStr::new("-o"), output_path.as_os_str()];
                if let Some(preload) = &preload {
                    args.push(OsStr::new("--preload"));
                    args.push(preload.as_os_str());
                }
                Ok(Command::new(ruvy).args(args).status()?)
            }
        }
    }

    fn wasi_args(&self, entrypoint: &Entrypoint) -> Vec<String> {
        match self {
            CompilationStrategy::WasiVFSRubyWasm => vec![
                // Not passing `--disable-gems` results in output about `RubyGems`,
                // `error_highlight`, `did_you_mean`, and `syntax_suggest` not
                // being loaded. We don't want that and we don't use the gems
                // anyway, so I'm disabling them.
                // If we did not want to pass `--disable-gems`, we can use the
                // `full` profile build of ruby.wasm and map the `/usr` directory.
                "--disable-gems".into(),
                PathBuf::from("/src")
                    .join(entrypoint.filename)
                    .to_string_lossy()
                    .to_string(),
            ],
            CompilationStrategy::Ruvy(_) => vec![],
        }
    }
}

impl Display for CompilationStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CompilationStrategy::WasiVFSRubyWasm => "rubywasm",
                CompilationStrategy::Ruvy(_) => "ruvy",
            }
        )
    }
}

struct Entrypoint<'a> {
    parent_dirname: &'a str,
    parent_path: &'a str,
    filename: &'a OsStr,
    input_path: PathBuf,
    path: &'a OsStr,
}

impl<'a> From<&'a str> for Entrypoint<'a> {
    fn from(value: &'a str) -> Self {
        let value = Path::new(value);
        Entrypoint {
            parent_dirname: value
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            parent_path: value.parent().unwrap().to_str().unwrap(),
            filename: value.file_name().unwrap(),
            input_path: value.parent().unwrap().join("input.json"),
            path: value.as_os_str(),
        }
    }
}
