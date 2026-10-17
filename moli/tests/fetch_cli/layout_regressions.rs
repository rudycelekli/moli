use super::clean_output;
use anyhow::{Context, Result, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::Value;
use std::{
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

fn run_layout(html: &str, expression: &str, fonts: bool) -> Result<Value> {
    let url = format!("data:text/html;base64,{}", STANDARD.encode(html));
    let mut command = Command::new(env!("CARGO_BIN_EXE_moli"));
    command.args([
        "fetch",
        "--log-level",
        "error",
        "--http-no-proxy",
        "*",
        "--layout",
        "--timeout",
        "2000",
        "--eval",
        expression,
    ]);
    if fonts {
        command.arg("--font");
    }
    let mut child = command
        .arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let started = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if started.elapsed() >= Duration::from_secs(10) {
            child.kill()?;
            let output = child.wait_with_output()?;
            bail!("layout did not terminate: {}", clean_output(&output.stderr));
        }
        thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output()?;
    assert!(output.status.success(), "{}", clean_output(&output.stderr));
    serde_json::from_slice(&output.stdout).context("layout probe must return JSON")
}

#[test]
fn fractional_grid_track_limits_terminate_with_a_finite_item_width() -> Result<()> {
    // This formerly loops in track growth even after the CLI's own timeout.
    // The subprocess deadline also bounds this regression if the loop returns.
    let html = r#"<!doctype html><style>
        body { margin: 0 }
        .grid { display: grid; width: 1000px; height: 100px;
            grid-template-columns: minmax(6.879453%, 20.244017%) }
        </style><div class="grid"><div id="item">test</div></div>"#;
    let value = run_layout(
        html,
        "document.querySelector('#item').getBoundingClientRect().width",
        false,
    )?;
    let width = value.as_f64().context("item width must be a number")?;
    assert!((width - 202.44017).abs() < 0.05, "{width}");
    Ok(())
}
