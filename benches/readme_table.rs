//! Reproduces the README performance table.
//!
//! ```text
//! cargo bench --bench readme_table
//! ```
//!
//! `RULE30_BENCH_STEPS` overrides the delay list (comma-separated powers of two).
//! `RULE30_CELLS` overrides the ring width (default 64, the CLI default).
//!
//! Work time is `build_trace` (the sequential Rule 30 evaluation). Proof
//! generation is Winterfell's STARK prover. Verify time is the median of 19
//! verifications after one warmup. Each delay is measured once for work and
//! proving. A trace of T rows attests to T - 1 Rule 30 transitions.

use std::env;
use std::fs;
use std::io::Write;
use std::process::Command;
use std::thread;
use std::time::Instant;

use winterfell::math::fields::f128::BaseElement;
use winterfell::math::FieldElement;

use rule30_vdf::air::Rule30PublicInputs;
use rule30_vdf::prover::{default_proof_options, verify_proof, Rule30Prover};
use rule30_vdf::trace::build_trace;
use rule30_vdf::DEFAULT_NUM_CELLS;

struct Row {
    steps: usize,
    work_secs: f64,
    prove_secs: f64,
    verify_median_secs: f64,
    verify_min_secs: f64,
    verify_max_secs: f64,
    proof_bytes: usize,
}

fn main() {
    let num_cells = env::var("RULE30_CELLS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_NUM_CELLS);
    let steps = bench_steps();
    let options = default_proof_options();

    println!("Rule 30 VDF benchmark");
    println!("  cells            : {num_cells}");
    println!("  steps            : {steps:?}");
    println!(
        "  proof options    : queries={} blowup={} grinding={} extension={:?}",
        options.num_queries(),
        options.blowup_factor(),
        options.grinding_factor(),
        options.field_extension()
    );
    println!("  machine          : {}", machine_summary());
    println!("  rustc            : {}", rustc_version());
    println!();

    let mut rows = Vec::with_capacity(steps.len());
    for num_steps in steps {
        assert!(
            num_steps.is_power_of_two() && num_steps >= 8,
            "delay {num_steps} is not a supported trace length"
        );
        rows.push(measure(num_cells, num_steps));
        let row = rows.last().expect("row was just pushed");
        println!(
            "  {:>6} steps | work {:8.3} s | prove {:8.3} s | verify median {:7.3} ms (min {:7.3} / max {:7.3}) | proof {} bytes",
            row.steps,
            row.work_secs,
            row.prove_secs,
            row.verify_median_secs * 1_000.0,
            row.verify_min_secs * 1_000.0,
            row.verify_max_secs * 1_000.0,
            row.proof_bytes
        );
    }

    let markdown = render_markdown(&rows);
    println!();
    println!("{markdown}");

    fs::create_dir_all("artifacts").expect("create artifacts/");
    let mut csv = fs::File::create("artifacts/benchmark.csv").expect("create benchmark.csv");
    writeln!(
        csv,
        "steps,work_seconds,prove_seconds,verify_median_seconds,verify_min_seconds,verify_max_seconds,proof_bytes"
    )
    .expect("write csv header");
    for row in &rows {
        writeln!(
            csv,
            "{},{:.6},{:.6},{:.6},{:.6},{:.6},{}",
            row.steps,
            row.work_secs,
            row.prove_secs,
            row.verify_median_secs,
            row.verify_min_secs,
            row.verify_max_secs,
            row.proof_bytes
        )
        .expect("write csv row");
    }

    let mut note = fs::File::create("artifacts/benchmark.md").expect("create benchmark.md");
    writeln!(note, "{markdown}").expect("write benchmark.md");
    writeln!(note).expect("write blank line");
    writeln!(note, "Machine: {}", machine_summary()).expect("write machine");
    writeln!(note, "rustc: {}", rustc_version()).expect("write rustc");
    writeln!(
        note,
        "Options: queries={} blowup={} grinding={} extension={:?} cells={num_cells}",
        options.num_queries(),
        options.blowup_factor(),
        options.grinding_factor(),
        options.field_extension()
    )
    .expect("write options");
    writeln!(
        note,
        "Method: one trace build, one proof, then 19 timed verifications after one warmup. Verify figures are the median."
    )
    .expect("write method");
}

fn bench_steps() -> Vec<usize> {
    match env::var("RULE30_BENCH_STEPS") {
        Ok(value) => value
            .split(',')
            .map(|part| {
                part.trim()
                    .parse()
                    .unwrap_or_else(|_| panic!("bad RULE30_BENCH_STEPS entry: {part}"))
            })
            .collect(),
        Err(_) => vec![1_024, 4_096, 16_384, 65_536],
    }
}

fn measure(num_cells: usize, num_steps: usize) -> Row {
    let mut init = vec![0u8; num_cells];
    init[num_cells / 2] = 1;

    let work_start = Instant::now();
    let (trace, final_state) = build_trace(&init, num_steps);
    let work_secs = work_start.elapsed().as_secs_f64();

    let init_state: Vec<BaseElement> = init
        .iter()
        .map(|&bit| {
            if bit == 0 {
                BaseElement::ZERO
            } else {
                BaseElement::ONE
            }
        })
        .collect();
    let pub_inputs = Rule30PublicInputs {
        init_state,
        final_state,
        num_cells,
    };
    let prover = Rule30Prover::new(default_proof_options(), pub_inputs.clone());

    let prove_start = Instant::now();
    let proof = prover.prove_trace(trace).expect("proof generation failed");
    let prove_secs = prove_start.elapsed().as_secs_f64();
    let proof_bytes = proof.to_bytes().len();

    verify_proof(proof.clone(), pub_inputs.clone()).expect("warmup verification failed");

    let mut samples = Vec::with_capacity(19);
    for _ in 0..19 {
        let sample_proof = proof.clone();
        let sample_inputs = pub_inputs.clone();
        let sample_start = Instant::now();
        let result = verify_proof(sample_proof, sample_inputs);
        let elapsed = sample_start.elapsed().as_secs_f64();
        result.expect("verification failed");
        samples.push(elapsed);
    }
    samples.sort_by(|left, right| left.partial_cmp(right).expect("verify times are finite"));

    Row {
        steps: num_steps,
        work_secs,
        prove_secs,
        verify_median_secs: samples[samples.len() / 2],
        verify_min_secs: samples[0],
        verify_max_secs: samples[samples.len() - 1],
        proof_bytes,
    }
}

fn render_markdown(rows: &[Row]) -> String {
    let mut out = String::from(
        "| Delay steps (trace rows) | Work time | Proof generation | Verify time (median) | Proof size |\n",
    );
    out.push_str(
        "|--------------------------|-----------|------------------|----------------------|------------|\n",
    );
    for row in rows {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            with_commas(row.steps),
            format_seconds(row.work_secs),
            format_seconds(row.prove_secs),
            format_seconds(row.verify_median_secs),
            format_kib(row.proof_bytes),
        ));
    }
    out
}

fn format_seconds(seconds: f64) -> String {
    if seconds < 1.0 {
        format!("{:.2} ms", seconds * 1_000.0)
    } else {
        format!("{seconds:.3} s")
    }
}

fn format_kib(bytes: usize) -> String {
    format!("{:.1} KB", bytes as f64 / 1024.0)
}

fn with_commas(value: usize) -> String {
    let raw = value.to_string();
    let mut out = String::new();
    for (index, ch) in raw.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn machine_summary() -> String {
    let model = fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|text| {
            text.lines()
                .find(|line| line.starts_with("model name"))
                .and_then(|line| line.split(':').nth(1))
                .map(|value| value.trim().to_string())
        })
        .unwrap_or_else(|| "unknown CPU".to_string());
    let mem_gib = fs::read_to_string("/proc/meminfo").ok().and_then(|text| {
        text.lines()
            .find(|line| line.starts_with("MemTotal:"))
            .and_then(|line| {
                line.split_whitespace()
                    .nth(1)
                    .and_then(|kb| kb.parse::<f64>().ok())
                    .map(|kb| kb / 1024.0 / 1024.0)
            })
    });
    let cpus = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    match mem_gib {
        Some(gib) => format!("{model}, {cpus} vCPU, {gib:.1} GiB RAM"),
        None => format!("{model}, {cpus} vCPU"),
    }
}

fn rustc_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}
