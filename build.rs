use std::process::Command;

fn git(args: &[&str]) -> String {
    let output = Command::new("git").args(args).output();

    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    }
}

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");

    println!(
        "cargo:rustc-env=GIT_COMMIT={}",
        git(&["rev-parse", "--short", "HEAD"])
    );

    println!(
        "cargo:rustc-env=GIT_BRANCH={}",
        git(&["rev-parse", "--abbrev-ref", "HEAD"])
    );

    let output = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .expect("failed to execute date");

    let build_time = String::from_utf8_lossy(&output.stdout).trim().to_string();

    println!("cargo:rustc-env=BUILD_TIME={build_time}");
}
