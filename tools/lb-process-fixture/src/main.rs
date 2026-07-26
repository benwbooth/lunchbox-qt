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
        "frontend-handoff" => write_arguments_from_env("LBPORT_FRONTEND_HANDOFF_LOG", &arguments),
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
        Some("pcsx2-saturated-card") => {
            write_pcsx2_saturated_card(&option_path(arguments, "--path")?)
        }
        Some("pcsx2-restore-source") => {
            write_pcsx2_restore_source(&option_path(arguments, "--path")?)
        }
        Some("pcsx2-extract") => {
            let card = option_path(arguments, "--card")?;
            let member = option(arguments, "--member")?
                .to_str()
                .ok_or_else(|| "--member must be Unicode".to_string())?;
            let destination = option_path(arguments, "--out")?;
            lb_integrations::pcsx2::extract_pcsx2_memory_card_save(&card, member, &destination)
                .map(|_| ())
                .map_err(|error| format!("extract PCSX2 fixture member: {error}"))
        }
        Some("pcsx2-disc-image") => {
            let path = option_path(arguments, "--path")?;
            let format = option(arguments, "--format")?
                .to_str()
                .ok_or_else(|| "--format must be Unicode".to_string())?;
            let serial = option(arguments, "--serial")?
                .to_str()
                .ok_or_else(|| "--serial must be Unicode".to_string())?;
            write_pcsx2_disc_image(&path, format, serial)
        }
        _ => Err(format!("unknown explicit fixture mode {mode:?}")),
    }
}

fn write_pcsx2_saturated_card(path: &Path) -> Result<(), String> {
    write_new_bytes(
        path,
        &lb_integrations::pcsx2::test_fixtures::saturated_raw_memory_card(),
        "PCSX2 saturated-card fixture",
    )
}

fn write_pcsx2_restore_source(path: &Path) -> Result<(), String> {
    fs::create_dir(path)
        .map_err(|error| io_error("create PCSX2 restore-source fixture directory", path, error))?;
    let result = (|| {
        let mut icon = vec![0_u8; 148];
        icon[..4].copy_from_slice(b"PS2D");
        icon[6..8].copy_from_slice(&7_u16.to_le_bytes());
        icon[80..95].copy_from_slice(b"Selected Member");
        write_new_bytes(&path.join("icon.sys"), &icon, "PCSX2 restore-source icon")?;
        write_new_bytes(
            &path.join("save.bin"),
            &vec![0x5a; lb_integrations::pcsx2::test_fixtures::CLUSTER_SIZE * 2 + 1],
            "PCSX2 restore-source save",
        )
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(path);
    }
    result
}

fn write_pcsx2_disc_image(path: &Path, format: &str, serial: &str) -> Result<(), String> {
    let bytes = match format {
        "iso" => lb_integrations::pcsx2::disc_test_fixtures::iso(serial),
        "gzip" => lb_integrations::pcsx2::disc_test_fixtures::gzip(serial),
        "cso" => lb_integrations::pcsx2::disc_test_fixtures::cso(serial),
        "chd-cd" => {
            if serial != "SLUS_203.12" {
                return Err("the authentic CHD fixture serial must be SLUS_203.12".into());
            }
            lb_integrations::pcsx2::disc_test_fixtures::chd_cd().to_vec()
        }
        "chd-dvd" => {
            if serial != "SLUS_203.12" {
                return Err("the authentic CHD fixture serial must be SLUS_203.12".into());
            }
            lb_integrations::pcsx2::disc_test_fixtures::chd_dvd().to_vec()
        }
        _ => {
            return Err(format!(
                "unsupported PCSX2 disc fixture format {format:?}; expected iso, gzip, cso, chd-cd, or chd-dvd"
            ))
        }
    };
    write_new_bytes(path, &bytes, "PCSX2 disc-image fixture")
}

fn write_new_bytes(path: &Path, bytes: &[u8], kind: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| io_error(&format!("create {kind}"), path, error))?;
    file.write_all(bytes)
        .map_err(|error| io_error(&format!("write {kind}"), path, error))?;
    file.sync_all()
        .map_err(|error| io_error(&format!("sync {kind}"), path, error))
}

fn executable_role() -> Result<String, String> {
    let executable = env::current_exe()
        .map_err(|error| format!("resolve current fixture executable: {error}"))?;
    let name = executable
        .file_stem()
        .and_then(OsStr::to_str)
        .ok_or_else(|| format!("fixture executable has a non-Unicode name: {executable:?}"))?
        .to_ascii_lowercase();
    let role = if name.contains("frontend-handoff-recorder") {
        "frontend-handoff"
    } else if name.contains("archive-recorder") {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pcsx2_fixture_modes_create_a_saturated_card_and_large_restore_source() {
        let directory = tempfile::tempdir().unwrap();
        let card = directory.path().join("Mcd001.ps2");
        run_explicit_mode(
            OsStr::new("pcsx2-saturated-card"),
            &[OsString::from("--path"), card.clone().into_os_string()],
        )
        .unwrap();
        let saves = lb_integrations::pcsx2::list_pcsx2_memory_card_saves(&card).unwrap();
        assert_eq!(saves.len(), 1);
        assert_eq!(saves[0].directory_name, "BASLUS-12345SAVE");
        assert!(run_explicit_mode(
            OsStr::new("pcsx2-saturated-card"),
            &[OsString::from("--path"), card.into_os_string()],
        )
        .is_err());

        let source = directory.path().join("source");
        run_explicit_mode(
            OsStr::new("pcsx2-restore-source"),
            &[OsString::from("--path"), source.clone().into_os_string()],
        )
        .unwrap();
        assert_eq!(
            fs::metadata(source.join("save.bin")).unwrap().len(),
            u64::try_from(lb_integrations::pcsx2::test_fixtures::CLUSTER_SIZE * 2 + 1).unwrap()
        );
        assert_eq!(&fs::read(source.join("icon.sys")).unwrap()[..4], b"PS2D");
    }

    #[test]
    fn pcsx2_extract_mode_routes_through_the_real_card_adapter() {
        let directory = tempfile::tempdir().unwrap();
        let card = directory.path().join("Mcd001.ps2");
        write_pcsx2_saturated_card(&card).unwrap();
        let output = directory.path().join("member");

        run_explicit_mode(
            OsStr::new("pcsx2-extract"),
            &[
                OsString::from("--card"),
                card.into_os_string(),
                OsString::from("--member"),
                OsString::from("BASLUS-12345SAVE"),
                OsString::from("--out"),
                output.clone().into_os_string(),
            ],
        )
        .unwrap();

        assert_eq!(fs::read(output.join("save.bin")).unwrap(), b"save bytes");
    }

    #[test]
    fn pcsx2_disc_image_mode_creates_images_readable_by_the_real_adapter() {
        let directory = tempfile::tempdir().unwrap();
        for (format, extension) in [
            ("iso", "iso"),
            ("gzip", "gz"),
            ("cso", "cso"),
            ("chd-cd", "cd.chd"),
            ("chd-dvd", "dvd.chd"),
        ] {
            let path = directory.path().join(format!("disc.{extension}"));
            run_explicit_mode(
                OsStr::new("pcsx2-disc-image"),
                &[
                    OsString::from("--path"),
                    path.clone().into_os_string(),
                    OsString::from("--format"),
                    OsString::from(format),
                    OsString::from("--serial"),
                    OsString::from("SLUS_203.12"),
                ],
            )
            .unwrap();
            assert_eq!(
                lb_integrations::pcsx2::extract_pcsx2_disc_serial(&path).as_deref(),
                Some("SLUS-20312")
            );
        }
    }
}
