//! The `news` help topic (`C-h n`): the latest changes, from the git history zett was built from.

fn main() {
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR"));
    let log = std::process::Command::new("git")
        .args(["log", "-60", "--date=short", "--format=- %ad  %s"])
        .output()
        .ok()
        .filter(|o| o.status.success() && !o.stdout.is_empty())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned());
    let text = match log {
        Some(l) => format!("# zett: what is new\n\nThe latest changes, newest first (from the history zett was built from).\n\n{l}"),
        None => "# zett: what is new\n\nThis zett was built without its git history, so there is no list of changes.\n".to_string(),
    };
    std::fs::write(out.join("news.md"), text).expect("news.md");
    // a new commit: a new list
    for p in ["../../.git/HEAD", "../../.git/refs/heads"] {
        println!("cargo:rerun-if-changed={p}");
    }
}
