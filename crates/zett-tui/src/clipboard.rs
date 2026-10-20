//! The system clipboard: wl-copy / wl-paste under Wayland, the terminal's OSC 52 otherwise.

use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// How long a paste may wait for the program that owns the clipboard.
const PASTE_WAIT: Duration = Duration::from_millis(300);

/// Puts `text` on the clipboard. False when nothing took it.
pub(crate) fn copy(text: &str) -> bool {
    if wayland() {
        if let Ok(mut c) = Command::new("wl-copy").stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn() {
            // through stdin: no length limit, and text starting with "-" stays text
            let wrote = c.stdin.take().is_some_and(|mut i| i.write_all(text.as_bytes()).is_ok());
            return wrote && c.wait().is_ok_and(|s| s.success());
        }
    }
    osc52(text)
}

/// The clipboard's text, when it can be read (Wayland only; a terminal cannot be asked safely).
pub(crate) fn paste() -> Option<String> {
    if !wayland() {
        return None;
    }
    let mut c = Command::new("wl-paste").args(["--no-newline", "--type", "text"]).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().ok()?;
    // a hung clipboard owner must not freeze the editor
    let start = Instant::now();
    loop {
        match c.try_wait() {
            Ok(Some(s)) if s.success() => break,
            Ok(Some(_)) | Err(_) => return None,
            Ok(None) if start.elapsed() > PASTE_WAIT => {
                let _ = c.kill();
                return None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
        }
    }
    let mut out = String::new();
    std::io::Read::read_to_string(&mut c.stdout.take()?, &mut out).ok()?;
    Some(out)
}

fn wayland() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_some_and(|v| !v.is_empty())
}

/// OSC 52: the terminal (zash, or one over SSH) puts it on its own clipboard.
fn osc52(text: &str) -> bool {
    let mut out = std::io::stdout();
    write!(out, "\x1b]52;c;{}\x07", base64(text.as_bytes())).and_then(|_| out.flush()).is_ok()
}

fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let n = c.iter().enumerate().fold(0u32, |n, (i, b)| n | (*b as u32) << (16 - 8 * i));
        for i in 0..4 {
            s.push(if i <= c.len() { T[(n >> (18 - 6 * i) & 63) as usize] as char } else { '=' });
        }
    }
    s
}

#[cfg(test)]
mod tests {
    #[test]
    fn base64_matches_rfc4648() {
        for (a, b) in [("", ""), ("f", "Zg=="), ("fo", "Zm8="), ("foo", "Zm9v"), ("foobar", "Zm9vYmFy")] {
            assert_eq!(super::base64(a.as_bytes()), b);
        }
    }
}
