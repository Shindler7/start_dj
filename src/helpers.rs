//! Helper utilities and supporting functions for the application.

use sysinfo::{Pid, System};
use std::process::id;

pub(crate) enum Shell {
    Sh,
    PowerShell,
    Cmd,
    Unknown,
}


pub(crate) fn detect_shell() -> Shell {
    let mut sys = System::new_all();
    sys.refresh_all();

    let current_pid = Pid::from(id() as usize);

    loop {
        let process = sys.process(current_pid).unwrap();
        let parent_pid = process.parent().unwrap();

        let parent_process = sys.process(parent_pid).unwrap();
        let parent_name = parent_process.name();
        println!("{:?}", parent_name);

        let current_pid = Pid::from(id() as usize);
    }

    if let Some(process) = sys.process(current_pid) {
        if let Some(parent_pid) = process.parent() {
            if let Some(parent_process) = sys.process(parent_pid) {
                let parent_name = parent_process.name();

                println!("{:?}", parent_name);

                if parent_name == "cargo.exe" {
                    if let Some(cargo_parent) = parent_process.parent() {
                        if let Some(parent_process) = sys.process(cargo_parent) {
                            let cargo_parent_name = parent_process.name();
                            println!("{:?}", cargo_parent_name);
                        }
                    }
                }


            }
        }
    }


    Shell::Unknown
}
