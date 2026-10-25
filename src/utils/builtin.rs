use std::path::PathBuf;

pub fn cd(arg: &str, oldpwd: &mut Option<PathBuf>) {
    tracing::debug!(arg = arg, "Running builtin cd");
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from);

    let target = match arg {
        "" | "~" => match home {
            Some(h) => h,
            None => return eprintln!("cd: HOME not set"),
        },
        "-" => match oldpwd.clone() {
            Some(p) => {
                println!("{}", p.display()); // bash prints the dir on `cd -`
                p
            }
            None => return eprintln!("cd: OLDPWD not set"),
        },
        _ => match (arg.strip_prefix("~/"), home) {
            (Some(rest), Some(h)) => h.join(rest),
            _ => PathBuf::from(arg),
        },
    };

    let prev = std::env::current_dir().ok();
    match std::env::set_current_dir(&target) {
        Ok(()) => {
            tracing::debug!(target = %target.display(), "Changed directory");
            *oldpwd = prev
        } // only update on success
        Err(e) => {
            tracing::debug!(target = %target.display(), error = %e, "Failed to change directory");
            eprintln!("cd: {}: {}", target.display(), e)
        }
    }
}
