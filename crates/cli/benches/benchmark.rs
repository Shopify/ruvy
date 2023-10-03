use std::{
    env::consts,
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

pub fn criterion_benchmark(c: &mut Criterion) {
    let engine = Engine::default();
    let cases = vec![
        WasmCase::new(
            BuildStrategy::WasiVFSRubyWasm,
            "benches/scripts/hello_world/hello_world.rb".into(),
        )
        .unwrap(),
        WasmCase::new(
            BuildStrategy::Ruvy(None),
            "benches/scripts/hello_world/hello_world.rb".into(),
        )
        .unwrap(),
        WasmCase::new(
            BuildStrategy::WasiVFSRubyWasm,
            "benches/scripts/transformer/ruby_wasm_entry.rb".into(),
        )
        .unwrap(),
        WasmCase::new(
            BuildStrategy::Ruvy(Some("benches/scripts/transformer/preload".into())),
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
    fn new(strategy: BuildStrategy, entrypoint: Entrypoint) -> Result<WasmCase> {
        let name = format!("{}-{}", strategy, entrypoint.parent_dirname);
        let output_path = cargo_target_tmpdir().join(&name).with_extension("wasm");
        let exit_status = strategy.build_wasm(&output_path, &entrypoint)?;
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

enum BuildStrategy {
    WasiVFSRubyWasm,
    Ruvy(Option<PathBuf>),
}

impl BuildStrategy {
    fn build_wasm(&self, output_path: &Path, entrypoint: &Entrypoint) -> Result<ExitStatus> {
        match self {
            Self::WasiVFSRubyWasm => {
                let ruby_wasm = ruby_wasm()?;
                let wasi_vfs = wasi_vfs()?;
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
            Self::Ruvy(preload) => {
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
            Self::WasiVFSRubyWasm => vec![
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
            Self::Ruvy(_) => vec![],
        }
    }
}

impl Display for BuildStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::WasiVFSRubyWasm => "rubywasm",
                Self::Ruvy(_) => "ruvy",
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

fn cargo_target_tmpdir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
}

fn ruby_wasm() -> Result<PathBuf> {
    let tmpdir = cargo_target_tmpdir();
    let ruby_wasm_base = ruby_wasm_assets::ruby_wasm_base_name();
    let ruby_wasm_dir = tmpdir.join(&ruby_wasm_base);
    let ruby_wasm = ruby_wasm_dir.join("usr/local/bin/ruby");
    if ruby_wasm.exists() {
        return Ok(ruby_wasm);
    }
    let archive = tmpdir.join(format!("{ruby_wasm_base}.tar.gz"));
    ruby_wasm_assets::download_ruby_wasm(&archive)?;
    ruby_wasm_assets::extract_tar(&archive, &ruby_wasm_dir, 1)?;
    Ok(ruby_wasm)
}

fn wasi_vfs() -> Result<PathBuf> {
    let tmpdir = cargo_target_tmpdir();
    const VERSION: &str = "0.4.0";
    let wasi_vfs_base = format!("wasi-vfs-{VERSION}");
    let directory = tmpdir.join(&wasi_vfs_base);
    let wasi_vfs = directory.join("wasi-vfs");
    if wasi_vfs.exists() {
        return Ok(wasi_vfs);
    }
    let archive = tmpdir.join(format!("{wasi_vfs_base}.tar.gz"));
    download_wasi_vfs(&archive, VERSION)?;
    extract_wasi_vfs(&archive, &directory)?;
    Ok(wasi_vfs)
}

fn download_wasi_vfs(path: &Path, version: &str) -> Result<()> {
    let file_suffix = match (consts::OS, consts::ARCH) {
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("windows", "x86_64") => "x86_64-pc-windows-gnu",
        other => bail!("Unsupported platform tuple {:?}", other),
    };
    ruby_wasm_assets::download(format!("https://github.com/kateinoigakukun/wasi-vfs/releases/download/v{version}/wasi-vfs-cli-{file_suffix}.zip"), path)
}

fn extract_wasi_vfs(archive: &Path, extract_to: &Path) -> Result<()> {
    let output = Command::new("unzip")
        .arg(archive)
        .arg("-d")
        .arg(extract_to)
        .output()?;
    if !output.status.success() {
        bail!(
            "Unpacking wasi-vfs failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
