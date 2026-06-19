// m4-rs CLI — forensic-parity GNU m4 command-line interface.
//
// WHO:   infinityabundance.
// WHAT:  Accepts GNU m4-compatible arguments, invokes the expansion engine,
//        handles EOF undivert, m4wrap, m4exit, and include paths.
// WHEN:  Binary entry point for m4-rs.
// WHERE: crates/m4-rs-cli/src/main.rs
// WHY:   CLI must match GNU m4 behavior for all admitted surfaces.
// HOW:   Parse args → configure engine → process inputs → flush → exit.

use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;

use m4_rs_core::expansion::ExpansionEngine;
use m4_rs_core::frozen;
use m4_rs_core::lexer::Lexer;

#[derive(Default)]
struct CliConfig {
    files: Vec<PathBuf>,
    defines: Vec<(String, Option<String>)>,
    undefines: Vec<String>,
    include_paths: Vec<PathBuf>,
    prefix_builtins: bool,
    synclines: bool,
    debug_flags: Option<String>,
    trace_names: Vec<String>,
    arg_length: Option<usize>,
    freeze_file: Option<PathBuf>,
    reload_file: Option<PathBuf>,
    traditional: bool,
    interactive: bool,
    fatal_warnings: bool,
    nesting_limit: Option<usize>,
    warn_syntax: bool,
    show_version: bool,
    show_help: bool,
}

fn main() {
    // Set C locale for byte-level parity with GNU m4
    std::env::set_var("LC_ALL", "C");

    let config = parse_args();

    if config.show_version {
        println!(
            "m4-rs {} (forensic-parity GNU m4)",
            env!("CARGO_PKG_VERSION")
        );
        process::exit(0);
    }
    if config.show_help {
        print_help();
        process::exit(0);
    }

    let mut engine = ExpansionEngine::new();
    engine.register_builtins();

    if config.prefix_builtins {
        engine.macro_table.prefix_builtins = true;
    }
    if config.synclines {
        engine.synclines = true;
    }
    if config.traditional {
        engine.macro_table.prefix_builtins = true;
    }
    if let Some(limit) = config.nesting_limit {
        engine.recursion_limit = limit;
    }
    if let Some(ref flags) = config.debug_flags {
        engine.debug_flags = flags.clone();
        if flags.contains('t') {
            engine.trace_names.insert(b"*".to_vec());
        }
    }
    for name in &config.trace_names {
        engine.trace_names.insert(name.as_bytes().to_vec());
    }

    // Apply -D and -U
    for (name, value) in &config.defines {
        let val = value
            .as_ref()
            .map(|v| v.as_bytes().to_vec())
            .unwrap_or_default();
        engine.macro_table.define(name.as_bytes(), &val);
    }
    for name in &config.undefines {
        engine.macro_table.undefine(name.as_bytes());
    }

    // Add include search paths
    for dir in &config.include_paths {
        engine.include_path.add(dir);
    }

    // Reload frozen state before processing input (-R)
    // GNU m4 loads frozen state BEFORE processing -D/-U and input files.
    // The frozen file contains macro definitions, pushdef stacks, quote/comment
    // settings, and diversion contents that were frozen at a previous exit.
    // Version mismatch in frozen file format causes exit code 63.
    if let Some(ref reload_path) = config.reload_file {
        match frozen::load_state(&mut engine, reload_path) {
            Ok(()) => {}
            Err(e) => {
                // GNU m4 exits with 63 on frozen file version mismatch
                if e.to_string().contains("unsupported frozen file version") {
                    eprintln!("m4-rs: {}: {}", reload_path.display(), e);
                    process::exit(63);
                }
                eprintln!("m4-rs: {}: {}", reload_path.display(), e);
                process::exit(1);
            }
        }
    }

    // Process input files or stdin
    if config.files.is_empty() {
        let data = read_stdin().unwrap_or_else(|e| {
            eprintln!("m4-rs: stdin: {}", e);
            process::exit(1);
        });
        process_input_owned(&mut engine, data, "<stdin>");
    } else {
        for file in &config.files {
            if file == &PathBuf::from("-") {
                let data = read_stdin().unwrap_or_else(|e| {
                    eprintln!("m4-rs: stdin: {}", e);
                    process::exit(1);
                });
                process_input_owned(&mut engine, data, "<stdin>");
            } else {
                match std::fs::read(file) {
                    Ok(data) => {
                        let name = file.to_string_lossy().to_string();
                        process_input_owned(&mut engine, data, &name);
                        if engine.exit_code.is_some() {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("m4-rs: {}: {}", file.display(), e);
                        process::exit(1);
                    }
                }
            }
        }
    }

    // EOF processing
    engine.undivert_all();
    engine.flush_wrap_buffer();

    // Output (exit gracefully on broken pipe)
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    if handle.write_all(&engine.output).is_err() {
        // Broken pipe or other write error — exit silently
        process::exit(0);
    }
    let _ = handle.flush();

    // Save frozen state after all processing and output (-F)
    // GNU m4 freezes state AFTER all input is processed, all diversions
    // are undiverted, and all m4wrap text is flushed. The frozen file
    // captures the final macro table, quote/comment settings, and
    // diversion contents for fast reload on subsequent runs.
    if let Some(ref freeze_path) = config.freeze_file {
        if let Err(e) = frozen::save_state(&engine, freeze_path) {
            eprintln!("m4-rs: {}: {}", freeze_path.display(), e);
            process::exit(1);
        }
    }

    if let Some(code) = engine.exit_code {
        process::exit(code);
    }
}

fn process_input_owned(engine: &mut ExpansionEngine, data: Vec<u8>, source_name: &str) {
    // Update engine's current file tracking for __file__ builtin
    engine.current_file = source_name.to_string();
    engine.emit_syncline(1, source_name);
    let mut lexer = Lexer::with_file(source_name.to_string());
    // Propagate the engine's quote/comment config (may have been loaded
    // from a frozen file or changed via changequote/changecom).
    lexer.quote_config = engine.quote_config.clone();
    let tokens = lexer.tokenize_owned(data);
    engine.expand_tokens(&tokens);
    // m4exit may have set exit_code during expansion
}

fn read_stdin() -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    io::stdin().lock().read_to_end(&mut data)?;
    Ok(data)
}

fn parse_args() -> CliConfig {
    let args: Vec<String> = std::env::args().collect();
    let mut config = CliConfig::default();
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "--version" => config.show_version = true,
            "--help" => config.show_help = true,
            "-P" | "--prefix-builtins" => config.prefix_builtins = true,
            "-s" | "--synclines" => config.synclines = true,
            "-D" => {
                i += 1;
                if i < args.len() {
                    let d = &args[i];
                    if let Some(eq) = d.find('=') {
                        config
                            .defines
                            .push((d[..eq].to_string(), Some(d[eq + 1..].to_string())));
                    } else {
                        config.defines.push((d.clone(), None));
                    }
                }
            }
            "-U" => {
                i += 1;
                if i < args.len() {
                    config.undefines.push(args[i].clone());
                }
            }
            "-I" => {
                i += 1;
                if i < args.len() {
                    config.include_paths.push(PathBuf::from(&args[i]));
                }
            }
            "-d" => {
                i += 1;
                if i < args.len() {
                    config.debug_flags = Some(args[i].clone());
                }
            }
            "-t" => {
                i += 1;
                if i < args.len() {
                    config.trace_names.push(args[i].clone());
                }
            }
            "-l" => {
                i += 1;
                if i < args.len() {
                    config.arg_length = args[i].parse().ok();
                }
            }
            "-F" => {
                i += 1;
                if i < args.len() {
                    config.freeze_file = Some(PathBuf::from(&args[i]));
                }
            }
            "-R" => {
                i += 1;
                if i < args.len() {
                    config.reload_file = Some(PathBuf::from(&args[i]));
                }
            }
            "-G" | "--traditional" => config.traditional = true,
            "-e" | "--interactive" => config.interactive = true,
            "-E" | "--fatal-warnings" => config.fatal_warnings = true,
            "-W" | "--warn-syntax" => config.warn_syntax = true,
            "-L" | "--nesting-limit" => {
                i += 1;
                if i < args.len() {
                    config.nesting_limit = args[i].parse().ok();
                }
            }
            "-" => config.files.push(PathBuf::from("-")),
            other => {
                if other.starts_with('-') && other.len() > 1 {
                    eprintln!("m4-rs: unrecognized option: {}", other);
                    process::exit(1);
                } else {
                    config.files.push(PathBuf::from(other));
                }
            }
        }
        i += 1;
    }
    config
}

fn print_help() {
    println!(
        "m4-rs {} — forensic-parity GNU m4",
        env!("CARGO_PKG_VERSION")
    );
    println!("Usage: m4-rs [OPTIONS] [FILES...]");
    println!("Options: --version --help -P -s -D -U -I -d -t -l -F -R");
    println!("WARNING: Not a GNU m4 replacement. See STATUS.md for admitted surfaces.");
}
