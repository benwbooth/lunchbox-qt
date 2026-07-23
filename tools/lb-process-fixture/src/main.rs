use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::Duration;

fn main() -> ExitCode {
    match run(env::args_os().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("lb-process-fixture: {message}");
            ExitCode::from(1)
        }
    }
}

fn run(mut arguments: Vec<OsString>) -> Result<(), String> {
    if arguments
        .first()
        .is_some_and(|value| value == "--fixture-mode")
    {
        if arguments.len() < 2 {
            return Err("--fixture-mode requires a value".into());
        }
        let mode = arguments.remove(1);
        arguments.remove(0);
        return run_explicit_mode(&mode, &arguments);
    }

    if let Some(marker) = env::var_os("LBPORT_UNEXPECTED_EXECUTION_MARKER") {
        fs::write(&marker, b"fixture executable was started\n")
            .map_err(|error| io_error("write unexpected-execution marker", &marker, error))?;
        return Err("an artifact marked as non-executable was started".into());
    }

    match executable_role()?.as_str() {
        "argument" => run_argument_recorder(&arguments),
        "archive" => run_archive_recorder(&arguments),
        "m3u" => run_m3u_recorder(&arguments),
        "sequence" => run_sequence_recorder(&arguments),
        "dosbox" => write_arguments_from_env("LBPORT_DOSBOX_ARGUMENT_LOG", &arguments),
        "scummvm" => write_arguments_from_env("LBPORT_SCUMMVM_ARGUMENT_LOG", &arguments),
        "noop" => Ok(()),
        role => Err(format!("unrecognized fixture executable role {role:?}")),
    }
}

fn run_explicit_mode(mode: &OsStr, arguments: &[OsString]) -> Result<(), String> {
    match mode.to_str() {
        Some("sleep") => {
            std::thread::sleep(Duration::from_millis(option_u64(
                arguments,
                "--duration-ms",
            )?));
            Ok(())
        }
        Some("paced-sleep") => {
            paced_sleep(option_u64(arguments, "--duration-ms")?);
            Ok(())
        }
        Some("exit") => {
            let code = option_u64(arguments, "--code")?;
            let code = u8::try_from(code).map_err(|_| "--code must fit u8".to_string())?;
            std::process::exit(i32::from(code));
        }
        Some("append") => {
            let path = option_path(arguments, "--path")?;
            let value = option(arguments, "--value")?;
            append_line(&path, value)
        }
        Some("delegate-sleep") => {
            let duration = option_u64(arguments, "--duration-ms")?;
            let child_pid = option_path(arguments, "--child-pid")?;
            spawn_sleep_child(duration, Some(&child_pid), false)
        }
        Some("delegate-paced") => {
            let duration = option_u64(arguments, "--duration-ms")?;
            let child_pid = option_path(arguments, "--child-pid")?;
            spawn_sleep_child(duration, Some(&child_pid), true)
        }
        Some("archive-after") => {
            let rom = option_path(arguments, "--rom")?;
            let directory = option_path(arguments, "--dir")?;
            let lifetime_log = option_path(arguments, "--lifetime-log")?;
            std::thread::sleep(Duration::from_millis(option_u64(
                arguments,
                "--duration-ms",
            )?));
            if !rom.is_file() || !directory.is_dir() {
                write_line(&lifetime_log, "missing-during-process")?;
                return Err("archive resource disappeared while delegated child ran".into());
            }
            write_line(&lifetime_log, "alive-until-exit")
        }
        Some("m3u-after") => {
            let playlist = option_path(arguments, "--playlist")?;
            let lifetime_log = option_path(arguments, "--lifetime-log")?;
            std::thread::sleep(Duration::from_millis(option_u64(
                arguments,
                "--duration-ms",
            )?));
            validate_playlist_members(&playlist).inspect_err(|_| {
                let _ = write_line(&lifetime_log, "missing-disc-during-process");
            })?;
            write_line(&lifetime_log, "alive-until-exit")
        }
        _ => Err(format!("unknown explicit fixture mode {mode:?}")),
    }
}

fn executable_role() -> Result<String, String> {
    let executable = env::current_exe()
        .map_err(|error| format!("resolve current fixture executable: {error}"))?;
    let name = executable
        .file_stem()
        .and_then(OsStr::to_str)
        .ok_or_else(|| format!("fixture executable has a non-Unicode name: {executable:?}"))?
        .to_ascii_lowercase();
    let role = if name.contains("archive-recorder") {
        "archive"
    } else if name.contains("m3u-recorder") {
        "m3u"
    } else if name.contains("sequence-recorder") {
        "sequence"
    } else if name.contains("dosbox-recorder") {
        "dosbox"
    } else if name == "scummvm" || name.contains("scummvm-recorder") {
        "scummvm"
    } else if name.contains("argument-recorder")
        || name.contains("fixture-emulator")
        || name.contains("edited-recorder")
    {
        "argument"
    } else if name.contains("archive-after") || name.contains("m3u-after") || name.contains("noop")
    {
        "noop"
    } else {
        return Err(format!("cannot infer a role from {name:?}"));
    };
    Ok(role.into())
}

fn run_argument_recorder(arguments: &[OsString]) -> Result<(), String> {
    write_arguments_from_env("LBPORT_LAUNCH_SMOKE_LOG", arguments)?;
    let seconds = env::var("LBPORT_LAUNCH_SMOKE_SLEEP")
        .unwrap_or_else(|_| "1.05".into())
        .parse::<f64>()
        .map_err(|error| format!("parse LBPORT_LAUNCH_SMOKE_SLEEP: {error}"))?;
    if !seconds.is_finite() || seconds.is_sign_negative() {
        return Err("LBPORT_LAUNCH_SMOKE_SLEEP must be a finite non-negative number".into());
    }
    let duration = Duration::from_secs_f64(seconds);
    if env::var_os("LBPORT_LAUNCH_SMOKE_DELEGATE").as_deref() == Some(OsStr::new("1")) {
        spawn_sleep_child(
            u64::try_from(duration.as_millis())
                .map_err(|_| "delegated sleep duration does not fit u64".to_string())?,
            None,
            false,
        )
    } else {
        std::thread::sleep(duration);
        Ok(())
    }
}

fn run_archive_recorder(arguments: &[OsString]) -> Result<(), String> {
    write_arguments_from_env("LBPORT_ARCHIVE_ARGUMENT_LOG", arguments)?;
    let lifetime_log = required_env_path("LBPORT_ARCHIVE_LIFETIME_LOG")?;
    let rom = argument_value(arguments, "--rom")?;
    let directory = argument_value(arguments, "--dir")?;
    if !rom.is_file() || !directory.is_dir() {
        write_line(&lifetime_log, "missing-before-exit")?;
        return Err("archive ROM or extraction directory is missing".into());
    }
    write_line(&lifetime_log, "alive-before-exit")?;
    spawn_current([
        OsString::from("--fixture-mode"),
        OsString::from("archive-after"),
        OsString::from("--duration-ms"),
        OsString::from("200"),
        OsString::from("--rom"),
        rom.into_os_string(),
        OsString::from("--dir"),
        directory.into_os_string(),
        OsString::from("--lifetime-log"),
        lifetime_log.into_os_string(),
    ])?;
    Ok(())
}

fn run_m3u_recorder(arguments: &[OsString]) -> Result<(), String> {
    write_arguments_from_env("LBPORT_M3U_ARGUMENT_LOG", arguments)?;
    let content_log = required_env_path("LBPORT_M3U_CONTENT_LOG")?;
    let lifetime_log = required_env_path("LBPORT_M3U_LIFETIME_LOG")?;
    let playlist = argument_value(arguments, "--playlist")?;
    let directory = argument_value(arguments, "--dir")?;
    if playlist.parent() != Some(directory.as_path()) {
        write_line(&lifetime_log, "missing-playlist-before-exit")?;
        return Err("playlist does not belong to the supplied directory".into());
    }
    validate_playlist_members(&playlist).inspect_err(|_| {
        let _ = write_line(&lifetime_log, "missing-disc-before-exit");
    })?;
    fs::copy(&playlist, &content_log)
        .map_err(|error| io_error("copy M3U content log", &content_log, error))?;
    write_line(&lifetime_log, "alive-before-exit")?;
    spawn_current([
        OsString::from("--fixture-mode"),
        OsString::from("m3u-after"),
        OsString::from("--duration-ms"),
        OsString::from("200"),
        OsString::from("--playlist"),
        playlist.into_os_string(),
        OsString::from("--lifetime-log"),
        lifetime_log.into_os_string(),
    ])?;
    Ok(())
}

fn run_sequence_recorder(arguments: &[OsString]) -> Result<(), String> {
    if arguments.len() != 2 || arguments[0] != "--step" {
        return Err(format!(
            "expected --step <value>, received {} argument(s)",
            arguments.len()
        ));
    }
    let log = required_env_path("LBPORT_SEQUENCE_SMOKE_LOG")?;
    append_line(&log, &arguments[1])
}

fn validate_playlist_members(playlist: &Path) -> Result<(), String> {
    let file = File::open(playlist)
        .map_err(|error| io_error("open generated playlist", playlist, error))?;
    let mut members = 0usize;
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|error| io_error("read generated playlist", playlist, error))?;
        if line.is_empty() || !Path::new(&line).is_file() {
            return Err(format!("playlist member is missing: {line:?}"));
        }
        members += 1;
    }
    if members == 0 {
        Err("generated playlist has no members".into())
    } else {
        Ok(())
    }
}

fn spawn_sleep_child(
    duration_ms: u64,
    child_pid: Option<&Path>,
    paced: bool,
) -> Result<(), String> {
    let child = spawn_current([
        OsString::from("--fixture-mode"),
        OsString::from(if paced { "paced-sleep" } else { "sleep" }),
        OsString::from("--duration-ms"),
        OsString::from(duration_ms.to_string()),
    ])?;
    if let Some(path) = child_pid {
        fs::write(path, child.to_string())
            .map_err(|error| io_error("write delegated child PID", path, error))?;
    }
    Ok(())
}

fn paced_sleep(duration_ms: u64) {
    for _ in 0..duration_ms / 10 {
        std::thread::sleep(Duration::from_millis(10));
    }
    let remainder = duration_ms % 10;
    if remainder > 0 {
        std::thread::sleep(Duration::from_millis(remainder));
    }
}

fn spawn_current<const N: usize>(arguments: [OsString; N]) -> Result<u32, String> {
    let executable = env::current_exe()
        .map_err(|error| format!("resolve current fixture executable: {error}"))?;
    Command::new(&executable)
        .args(arguments)
        .spawn()
        .map(|child| child.id())
        .map_err(|error| format!("spawn delegated fixture {executable:?}: {error}"))
}

fn write_arguments_from_env(variable: &str, arguments: &[OsString]) -> Result<(), String> {
    let path = required_env_path(variable)?;
    let mut file =
        File::create(&path).map_err(|error| io_error("create argument log", &path, error))?;
    for argument in arguments {
        writeln!(file, "{}", argument.to_string_lossy())
            .map_err(|error| io_error("write argument log", &path, error))?;
    }
    file.sync_all()
        .map_err(|error| io_error("sync argument log", &path, error))
}

fn required_env_path(variable: &str) -> Result<PathBuf, String> {
    env::var_os(variable)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| format!("{variable} is required"))
}

fn argument_value(arguments: &[OsString], key: &str) -> Result<PathBuf, String> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == key)
        .map(|pair| PathBuf::from(&pair[1]))
        .ok_or_else(|| format!("{key} requires a value"))
}

fn option<'a>(arguments: &'a [OsString], key: &str) -> Result<&'a OsStr, String> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == key)
        .map(|pair| pair[1].as_os_str())
        .ok_or_else(|| format!("{key} requires a value"))
}

fn option_path(arguments: &[OsString], key: &str) -> Result<PathBuf, String> {
    option(arguments, key).map(PathBuf::from)
}

fn option_u64(arguments: &[OsString], key: &str) -> Result<u64, String> {
    option(arguments, key)?
        .to_str()
        .ok_or_else(|| format!("{key} must be Unicode"))?
        .parse()
        .map_err(|error| format!("parse {key}: {error}"))
}

fn write_line(path: &Path, value: &str) -> Result<(), String> {
    let mut file =
        File::create(path).map_err(|error| io_error("create fixture log", path, error))?;
    writeln!(file, "{value}").map_err(|error| io_error("write fixture log", path, error))?;
    file.sync_all()
        .map_err(|error| io_error("sync fixture log", path, error))
}

fn append_line(path: &Path, value: &OsStr) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| io_error("open append log", path, error))?;
    writeln!(file, "{}", value.to_string_lossy())
        .map_err(|error| io_error("append fixture log", path, error))?;
    file.sync_all()
        .map_err(|error| io_error("sync append log", path, error))
}

fn io_error(operation: &str, path: impl AsRef<Path>, error: std::io::Error) -> String {
    format!("{operation} at {}: {error}", path.as_ref().display())
}
