use crate::domain::ConvertError;
use std::{
    io::{self, BufRead, BufReader, Read},
    process::{ChildStdout, Command, Output, Stdio},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    thread,
    time::Duration,
};

pub fn run(program: &str, args: &[String]) -> Result<Output, ConvertError> {
    run_cancellable(program, args, &AtomicBool::new(false))
}

pub fn run_cancellable(
    program: &str,
    args: &[String],
    cancelled: &AtomicBool,
) -> Result<Output, ConvertError> {
    run_child(
        program,
        args,
        cancelled,
        |mut stream| {
            let mut bytes = Vec::new();
            let mut buffer = [0; 8192];
            let mut exceeded = false;
            loop {
                let count = stream.read(&mut buffer)?;
                if count == 0 {
                    break;
                }
                // Keep draining the pipe even when rejecting unexpectedly large output.
                if bytes.len() + count <= 16 * 1024 * 1024 {
                    bytes.extend_from_slice(&buffer[..count]);
                } else {
                    exceeded = true;
                }
            }
            if exceeded {
                return Err(io::Error::other("Вывод процесса превышает 16 МБ"));
            }
            Ok(bytes)
        },
        || {},
    )
}

pub fn run_with_progress<F>(
    program: &str,
    args: &[String],
    duration_seconds: Option<f64>,
    cancelled: &AtomicBool,
    mut on_progress: F,
) -> Result<Output, ConvertError>
where
    F: FnMut(f32),
{
    let mut command_args = vec!["-progress".into(), "pipe:1".into(), "-nostats".into()];
    command_args.extend_from_slice(args);
    let position = AtomicU64::new(0);
    let mut previous = u64::MAX;
    run_child(
        program,
        &command_args,
        cancelled,
        |stream| {
            for line in BufReader::new(stream).lines() {
                if let Some(value) = line?
                    .strip_prefix("out_time_us=")
                    .and_then(|text| text.trim().parse::<u64>().ok())
                {
                    position.store(value, Ordering::Relaxed);
                }
            }
            Ok(Vec::new())
        },
        || {
            let current = position.load(Ordering::Relaxed);
            if current != previous {
                previous = current;
                if let Some(duration) =
                    duration_seconds.filter(|value| value.is_finite() && *value > 0.0)
                {
                    on_progress(
                        (current as f64 / 1_000_000.0 / duration * 100.0).clamp(0.0, 99.5) as f32,
                    );
                }
            }
        },
    )
}

fn run_child<R, F>(
    program: &str,
    args: &[String],
    cancelled: &AtomicBool,
    read_stdout: R,
    mut tick: F,
) -> Result<Output, ConvertError>
where
    R: FnOnce(ChildStdout) -> io::Result<Vec<u8>> + Send,
    F: FnMut(),
{
    if cancelled.load(Ordering::Relaxed) {
        return Err(ConvertError::Cancelled);
    }
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // The parent handles Ctrl+C and cleanup; no extra console for FFmpeg.
        command.creation_flags(0x08000000);
    }
    let mut child = command
        .spawn()
        .map_err(|error| ConvertError::ProcessFailed(error.to_string()))?;
    let stdout = child.stdout.take().expect("piped stdout");
    let mut stderr = child.stderr.take().expect("piped stderr");
    thread::scope(|scope| {
        let stdout = scope.spawn(move || read_stdout(stdout));
        let stderr = scope.spawn(move || {
            let mut tail = Vec::new();
            let mut buffer = [0; 4096];
            loop {
                let count = stderr.read(&mut buffer)?;
                if count == 0 {
                    break;
                }
                tail.extend_from_slice(&buffer[..count]);
                if tail.len() > 64 * 1024 {
                    tail.drain(..tail.len() - 64 * 1024);
                }
            }
            Ok::<_, io::Error>(tail)
        });
        let status = loop {
            if cancelled.load(Ordering::Relaxed) {
                let _ = child.kill();
                break child.wait();
            }
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) => {
                    tick();
                    thread::sleep(Duration::from_millis(50));
                }
                Err(error) => break Err(error),
            }
        };
        if status.is_err() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let stdout = stdout.join();
        let stderr = stderr.join();
        if cancelled.load(Ordering::Relaxed) {
            return Err(ConvertError::Cancelled);
        }
        tick();
        let process_error = |error: io::Error| ConvertError::ProcessFailed(error.to_string());
        Ok(Output {
            status: status.map_err(process_error)?,
            stdout: stdout
                .map_err(|_| ConvertError::ProcessFailed("Ошибка чтения stdout".into()))?
                .map_err(process_error)?,
            stderr: stderr
                .map_err(|_| ConvertError::ProcessFailed("Ошибка чтения stderr".into()))?
                .map_err(process_error)?,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Requires FFmpeg; run with --include-ignored"]
    fn cancellation_does_not_wait_for_progress_output() {
        let ffmpeg = crate::infra::bin_resolver::resolve("ffmpeg").unwrap();
        let cancelled = AtomicBool::new(false);
        let started = std::time::Instant::now();
        thread::scope(|scope| {
            scope.spawn(|| {
                thread::sleep(Duration::from_millis(300));
                cancelled.store(true, Ordering::Relaxed);
            });
            let result = run_with_progress(
                &ffmpeg.to_string_lossy(),
                &[
                    "-nostdin".into(),
                    "-loglevel".into(),
                    "error".into(),
                    "-stats_period".into(),
                    "100".into(),
                    "-re".into(),
                    "-f".into(),
                    "lavfi".into(),
                    "-i".into(),
                    "sine=duration=30".into(),
                    "-f".into(),
                    "null".into(),
                    "-".into(),
                ],
                Some(30.0),
                &cancelled,
                |_| {},
            );
            assert!(matches!(result, Err(ConvertError::Cancelled)));
        });
        assert!(started.elapsed() < Duration::from_secs(5));
    }
}
