//! Emacs's hexl-mode: a file's bytes as a buffer (offset, 16 bytes in hex, their characters), edited like text;
//! :w turns the hex columns back into bytes and writes the file (the character column is only shown, then redrawn).
use super::*;

/// Lines like `00000010: 4865 6c6c 6f0a ...  Hello.`
pub(crate) fn dump(bytes: &[u8]) -> String {
    let mut out = Vec::new();
    for (i, chunk) in bytes.chunks(16).enumerate() {
        let mut hex = String::new();
        for (j, b) in chunk.iter().enumerate() {
            hex.push_str(&format!("{b:02x}"));
            if j % 2 == 1 {
                hex.push(' ');
            }
        }
        let chars: String = chunk.iter().map(|&b| if (0x20..0x7f).contains(&b) { b as char } else { '.' }).collect();
        out.push(format!("{:08x}: {:<40} {chars}", i * 16, hex));
    }
    out.join("\n")
}

/// The bytes the hex columns hold; or the line that is not hex.
pub(crate) fn parse(text: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let rest = line.split_once(": ").map(|(_, r)| r).ok_or(format!("line {}: no offset", n + 1))?;
        // the hex ends where the character column starts (two spaces, or the 40 columns)
        let hex = rest.get(..40.min(rest.len())).unwrap_or(rest);
        let hex = hex.split("  ").next().unwrap_or("");
        let digits: String = hex.chars().filter(|c| !c.is_whitespace()).collect();
        if digits.len() % 2 == 1 {
            return Err(format!("line {}: an odd number of hex digits", n + 1));
        }
        for i in (0..digits.len()).step_by(2) {
            bytes.push(u8::from_str_radix(&digits[i..i + 2], 16).map_err(|_| format!("line {}: \"{}\" is not hex", n + 1, &digits[i..i + 2]))?);
        }
    }
    Ok(bytes)
}

impl App {
    /// M-x hexl-find-file, :Hexl [file]: the file (here: the current one) as hex.
    pub(crate) fn hexl_open(&mut self, file: Option<PathBuf>) {
        let Some(path) = file.map(|f| if f.is_absolute() { f } else { self.root.join(f) }).or_else(|| self.docs[self.cur].abs_path()) else {
            return self.say("hexl: no file");
        };
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => return self.say(format!("{}: {e}", path.display())),
        };
        let name = format!("*hexl {}*", path.display());
        if let Some(i) = self.docs.iter().position(|d| d.name == name) {
            self.docs.remove(i);
        }
        let mut d = Doc::new(Editor::from_str(&dump(&bytes)), name.clone(), "Plain Text".into());
        d.local.insert("buftype".into(), "acwrite".into());
        self.docs.push(d);
        self.hexl.insert(name, path);
        let i = self.docs.len() - 1;
        self.show_doc(i);
        self.say(format!("{} bytes · edit the hex, :w writes them", bytes.len()));
    }

    /// :w in a hex buffer: the bytes to the file, the dump drawn again.
    pub(crate) fn hexl_save(&mut self) {
        let name = self.docs[self.cur].name.clone();
        let Some(path) = self.hexl.get(&name).cloned() else { return };
        let bytes = match parse(&self.docs[self.cur].ed.text()) {
            Ok(b) => b,
            Err(e) => return self.say(format!("not written: {e}")),
        };
        if let Err(e) = std::fs::write(&path, &bytes) {
            return self.say(format!("{}: {e}", path.display()));
        }
        let cursor = self.docs[self.cur].ed.cursor();
        self.hexl_open(Some(path.clone()));
        let ed = &mut self.docs[self.cur].ed;
        let line = cursor.line.min(ed.buf.len_lines().saturating_sub(1));
        ed.set_cursor(zett_core::Pos { line, col: cursor.col });
        self.say(format!("{}: {} bytes written", path.display(), bytes.len()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dump_and_back() {
        let b: Vec<u8> = (0u8..=255).chain(b"tail".iter().copied()).collect();
        let d = dump(&b);
        assert!(d.starts_with("00000000: 0001 0203"), "{d}");
        assert!(d.lines().nth(2).unwrap().ends_with(" !\"#$%&'()*+,-./"), "{d}");
        assert_eq!(parse(&d).unwrap(), b);
        // an edited digit changes that byte; the character column does not count
        let e = d.replacen("0001", "ff01", 1).replacen("Hello", "xxxxx", 1);
        assert_eq!(parse(&e).unwrap()[0], 0xff);
        assert!(parse("00000000: 4g").is_err());
    }

    #[test]
    fn saving_writes_the_bytes() {
        let dir = std::env::temp_dir().join(format!("zett-hexl-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("a.bin");
        std::fs::write(&f, b"Hi\n").unwrap();
        let mut app = crate::App::new(Vec::new());
        app.hexl_open(Some(f.clone()));
        assert_eq!(app.docs[app.cur].ed.text(), format!("00000000: {:<40} Hi.", "4869 0a"));
        app.command("s/4869/4849/");
        app.command("w");
        assert_eq!(std::fs::read(&f).unwrap(), b"HI\n");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
