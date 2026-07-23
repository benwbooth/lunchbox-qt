use lb_platform::{
    execute_launch_sequence_controlled, execute_launch_sequence_with, LaunchControlCommand,
    LaunchKind, LaunchPausePolicy, LaunchPlan, LaunchProcess, LaunchSequence, LaunchSequenceEvent,
    LaunchShutdownPolicy, LaunchStartupPolicy, LaunchStartupSettingsSource, LaunchStep,
    LaunchStepRole, LaunchTarget, ProcessLauncher, SystemProcessLauncher,
};
use std::ffi::OsString;
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn fixture_executable() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_lb-process-fixture"))
}

fn fixture_request(mode: &str) -> lb_platform::LaunchRequest {
    lb_platform::LaunchRequest::new(fixture_executable())
        .arg("--fixture-mode")
        .arg(mode)
}

fn primary_sequence(request: lb_platform::LaunchRequest) -> LaunchSequence {
    LaunchSequence {
        game_id: "fixture-game".into(),
        game_title: "Fixture Game".into(),
        startup: LaunchStartupPolicy::disabled(),
        shutdown: LaunchShutdownPolicy::disabled(),
        pause: LaunchPausePolicy {
            enabled: true,
            suspend_process: true,
            forceful_activation: false,
            source: LaunchStartupSettingsSource::DirectGame,
        },
        steps: vec![LaunchStep {
            role: LaunchStepRole::MainGame,
            wait_for_exit: false,
            plan: LaunchPlan {
                game_id: "fixture-game".into(),
                game_title: "Fixture Game".into(),
                target: LaunchTarget::MainGame,
                kind: LaunchKind::Direct,
                request,
                resource_leases: Vec::new(),
            },
        }],
    }
}

fn wait_for_control_event(receiver: &mpsc::Receiver<LaunchSequenceEvent>, expected_pause: bool) {
    loop {
        let event = receiver
            .recv_timeout(Duration::from_secs(3))
            .expect("receive process-control event");
        let matched = if expected_pause {
            matches!(
                event,
                LaunchSequenceEvent::PrimaryPaused {
                    process_suspended: true
                }
            )
        } else {
            matches!(
                event,
                LaunchSequenceEvent::PrimaryResumed {
                    process_resumed: true
                }
            )
        };
        if matched {
            return;
        }
    }
}

#[test]
fn system_launcher_executes_the_portable_fixture_without_a_shell() {
    let mut process = SystemProcessLauncher
        .launch(&fixture_request("exit").arg("--code").arg("0"))
        .expect("launch portable fixture");
    assert!(process.wait().expect("wait for portable fixture").success());
}

#[test]
fn sequence_executor_preserves_waited_before_main_and_after_order() {
    let temporary = tempfile::tempdir().expect("create sequence fixture");
    let log = temporary.path().join("sequence.log");
    let append = |value: &str| {
        fixture_request("append")
            .arg("--path")
            .arg(log.as_os_str())
            .arg("--value")
            .arg(value)
    };
    let step = |role, wait_for_exit, name: &str| LaunchStep {
        role,
        wait_for_exit,
        plan: LaunchPlan {
            game_id: "fixture-game".into(),
            game_title: "Fixture Game".into(),
            target: if role == LaunchStepRole::MainGame {
                LaunchTarget::MainGame
            } else {
                LaunchTarget::AdditionalApplication {
                    application_id: name.into(),
                    application_name: name.into(),
                }
            },
            kind: LaunchKind::Direct,
            request: append(name),
            resource_leases: Vec::new(),
        },
    };
    let sequence = LaunchSequence {
        game_id: "fixture-game".into(),
        game_title: "Fixture Game".into(),
        startup: LaunchStartupPolicy::disabled(),
        shutdown: LaunchShutdownPolicy::disabled(),
        pause: LaunchPausePolicy::disabled(),
        steps: vec![
            step(LaunchStepRole::AutomaticBefore, true, "before"),
            step(LaunchStepRole::MainGame, true, "main"),
            step(LaunchStepRole::AutomaticAfter, false, "after"),
        ],
    };
    let mut events = Vec::new();
    let report = execute_launch_sequence_with(
        &sequence,
        &SystemProcessLauncher,
        Duration::from_millis(100),
        |event| events.push(event),
    )
    .expect("execute portable sequence");

    assert_eq!(report.automatic_before_started, 1);
    assert_eq!(report.automatic_after_started, 1);
    assert_eq!(report.before_wait_timeouts, 0);
    assert!(!report.delegated_descendant_observed);
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, LaunchSequenceEvent::StepStarted { .. }))
            .count(),
        3
    );
    let started = Instant::now();
    let contents = loop {
        let contents = std::fs::read_to_string(&log).unwrap_or_default();
        if contents.lines().count() == 3 {
            break contents;
        }
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "after-app did not finish: {contents:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(
        contents.lines().collect::<Vec<_>>(),
        ["before", "main", "after"]
    );
}

#[test]
fn before_wait_timeout_does_not_prevent_the_primary_fixture() {
    let before = LaunchStep {
        role: LaunchStepRole::AutomaticBefore,
        wait_for_exit: true,
        plan: LaunchPlan {
            game_id: "fixture-game".into(),
            game_title: "Fixture Game".into(),
            target: LaunchTarget::AdditionalApplication {
                application_id: "slow-before".into(),
                application_name: "Slow Before".into(),
            },
            kind: LaunchKind::Direct,
            request: fixture_request("sleep").arg("--duration-ms").arg("50"),
            resource_leases: Vec::new(),
        },
    };
    let mut sequence = primary_sequence(
        fixture_request("exit")
            .arg("--code")
            .arg(OsString::from("0")),
    );
    sequence.pause = LaunchPausePolicy::disabled();
    sequence.steps.insert(0, before);
    let mut timed_out = false;
    let report = execute_launch_sequence_with(
        &sequence,
        &SystemProcessLauncher,
        Duration::from_millis(1),
        |event| timed_out |= matches!(event, LaunchSequenceEvent::BeforeWaitTimedOut { .. }),
    )
    .expect("timeout remains non-fatal");

    assert!(timed_out);
    assert_eq!(report.before_wait_timeouts, 1);
    assert_ne!(report.primary_pid, 0);
}

#[test]
fn controlled_executor_pauses_and_resumes_the_direct_portable_fixture() {
    let sequence = primary_sequence(
        fixture_request("paced-sleep")
            .arg("--duration-ms")
            .arg("600"),
    );
    let (command_tx, command_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::channel();
    let started = Instant::now();
    let worker = std::thread::spawn(move || {
        execute_launch_sequence_controlled(&sequence, &command_rx, |event| {
            event_tx.send(event).expect("send direct fixture event");
        })
    });

    loop {
        if matches!(
            event_rx
                .recv_timeout(Duration::from_secs(3))
                .expect("receive direct start event"),
            LaunchSequenceEvent::StepStarted { role, .. } if role.is_primary()
        ) {
            break;
        }
    }
    command_tx
        .send(LaunchControlCommand::Pause)
        .expect("pause direct fixture");
    wait_for_control_event(&event_rx, true);
    std::thread::sleep(Duration::from_millis(300));
    command_tx
        .send(LaunchControlCommand::Resume)
        .expect("resume direct fixture");
    wait_for_control_event(&event_rx, false);
    drop(command_tx);
    let report = worker
        .join()
        .expect("join direct fixture worker")
        .expect("supervise direct fixture");

    assert!(report.primary_exit_success);
    assert!(!report.delegated_descendant_observed);
    assert!(report.primary_runtime >= Duration::from_millis(800));
    assert!(started.elapsed() >= Duration::from_millis(800));
}

#[test]
fn controlled_executor_supervises_and_pauses_a_delegated_portable_fixture() {
    let temporary = tempfile::tempdir().expect("create delegated fixture");
    let child_pid = temporary.path().join("child.pid");
    let sequence = primary_sequence(
        fixture_request("delegate-paced")
            .arg("--duration-ms")
            .arg("650")
            .arg("--child-pid")
            .arg(child_pid.as_os_str()),
    );
    let (command_tx, command_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::channel();
    let started = Instant::now();
    let worker = std::thread::spawn(move || {
        execute_launch_sequence_controlled(&sequence, &command_rx, |event| {
            event_tx.send(event).expect("send delegated fixture event");
        })
    });

    loop {
        if matches!(
            event_rx
                .recv_timeout(Duration::from_secs(3))
                .expect("receive delegating start event"),
            LaunchSequenceEvent::StepStarted { role, .. } if role.is_primary()
        ) {
            break;
        }
    }
    let published = Instant::now();
    while !child_pid.is_file() {
        assert!(
            published.elapsed() < Duration::from_secs(2),
            "delegating fixture did not publish its child PID"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    std::thread::sleep(Duration::from_millis(100));
    command_tx
        .send(LaunchControlCommand::Pause)
        .expect("pause delegated fixture");
    wait_for_control_event(&event_rx, true);
    std::thread::sleep(Duration::from_millis(300));
    command_tx
        .send(LaunchControlCommand::Resume)
        .expect("resume delegated fixture");
    wait_for_control_event(&event_rx, false);
    drop(command_tx);
    let report = worker
        .join()
        .expect("join delegated fixture worker")
        .expect("supervise delegated fixture");

    assert!(report.primary_exit_success);
    assert!(report.delegated_descendant_observed);
    assert!(report.primary_runtime >= Duration::from_millis(850));
    assert!(started.elapsed() >= Duration::from_millis(850));
}
