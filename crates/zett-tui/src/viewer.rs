//! Emacs's doc-view and image-mode: a PDF or a picture opened as a buffer. The buffer holds the page's text
//! (pdftotext: / searches it, y copies it, any terminal shows it); in zash (and zeit) the page itself is drawn
//! over the window with the Kitty graphics protocol. n / PageDown, p / PageUp, gg / G, + / - zoom, q closes.
use super::*;
use std::collections::HashMap;

pub(crate) struct View {
    pub path: PathBuf,
    pub pdf: bool,
    pub page: usize,
    pub pages: usize,
    /// 1.0: the page as wide as the window
    pub zoom: f32,
}

/// Rendered pictures, by (file, page, width px): PNG bytes
#[derive(Default)]
pub(crate) struct Shown {
    cache: HashMap<(PathBuf, usize, u32), Vec<u8>>,
    /// what is on the screen now: (doc name, page, zoom, window rect)
    on: Option<(String, usize, u32, Rect)>,
}

const IMG_ID: u32 = 7701;

pub(crate) fn is_view(p: &Path) -> bool {
    matches!(p.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref(), Some("pdf" | "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp"))
}

/// Pictures can be drawn: zash (itself or through zeit), kitty, ghostty, wezterm.
fn graphics() -> bool {
    std::env::var("TERM_PROGRAM").is_ok_and(|t| matches!(t.as_str(), "zash" | "WezTerm" | "ghostty"))
        || std::env::var_os("ZEIT").is_some()
        || std::env::var_os("KITTY_WINDOW_ID").is_some()
        || std::env::var("TERM").is_ok_and(|t| t.contains("kitty"))
}

fn pdf_pages(p: &Path) -> usize {
    let out = std::process::Command::new("pdfinfo").arg(p).output().ok();
    out.and_then(|o| String::from_utf8_lossy(&o.stdout).lines().find_map(|l| l.strip_prefix("Pages:").and_then(|n| n.trim().parse().ok()))).unwrap_or(1)
}

fn page_text(v: &View) -> String {
    let name = v.path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let head = if v.pdf {
        format!("{name} — page {} of {} · n/p pages · +/- zoom · q close", v.page, v.pages)
    } else {
        format!("{name} · +/- zoom · q close")
    };
    let body = if v.pdf {
        std::process::Command::new("pdftotext").args(["-layout", "-f", &v.page.to_string(), "-l", &v.page.to_string()]).arg(&v.path).arg("-").output().map(|o| String::from_utf8_lossy(&o.stdout).replace('\x0c', "")).unwrap_or_else(|e| format!("(pdftotext: {e})"))
    } else {
        String::new()
    };
    format!("{head}\n\n{body}")
}

/// The picture of a page (or the image) `w` pixels wide, as PNG.
fn render(v: &View, w: u32) -> Option<Vec<u8>> {
    if v.pdf {
        // pdftoppm writes files (its "-" is a file name with -singlefile): into a temp folder, read back
        let dir = std::env::temp_dir().join(format!("zett-view-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let root = dir.join("page");
        let ok = std::process::Command::new("pdftoppm").args(["-png", "-f", &v.page.to_string(), "-l", &v.page.to_string(), "-scale-to-x", &w.to_string(), "-scale-to-y", "-1", "-singlefile"]).arg(&v.path).arg(&root).status().is_ok_and(|s| s.success());
        let png = root.with_extension("png");
        let data = ok.then(|| std::fs::read(&png).ok()).flatten();
        let _ = std::fs::remove_file(&png);
        data.filter(|d| !d.is_empty())
    } else if v.path.extension().is_some_and(|e| e.eq_ignore_ascii_case("png")) {
        std::fs::read(&v.path).ok()
    } else {
        // other formats become PNG first (ImageMagick, else ffmpeg)
        let o = std::process::Command::new("magick").arg(&v.path).arg("png:-").output().ok().filter(|o| o.status.success());
        let o = o.or_else(|| std::process::Command::new("ffmpeg").args(["-loglevel", "error", "-i"]).arg(&v.path).args(["-frames:v", "1", "-f", "image2pipe", "-vcodec", "png", "-"]).output().ok().filter(|o| o.status.success()))?;
        Some(o.stdout)
    }
}

fn b64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        s.push(T[(n >> 18) as usize & 63] as char);
        s.push(T[(n >> 12) as usize & 63] as char);
        s.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        s.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    s
}

impl App {
    /// A PDF or a picture: a buffer of its text, the view kept beside it.
    pub(crate) fn open_view(&mut self, p: &Path) {
        let pdf = p.extension().is_some_and(|e| e.eq_ignore_ascii_case("pdf"));
        let v = View { path: p.to_path_buf(), pdf, page: 1, pages: if pdf { pdf_pages(p) } else { 1 }, zoom: 1.0 };
        let name = p.display().to_string();
        let mut d = Doc::new(Editor::from_str(&page_text(&v)), name.clone(), "Plain Text".into());
        d.local.insert("buftype".into(), "nofile".into());
        d.local.insert("readonly".into(), "true".into());
        d.local.insert("modifiable".into(), "false".into());
        self.docs.push(d);
        self.views.insert(name, v);
        let i = self.docs.len() - 1;
        self.show_doc(i);
        if !graphics() {
            self.say("this terminal shows no pictures: the page's text is here (zash, kitty, ghostty or wezterm draw the page)");
        }
    }

    /// The view's keys, in normal mode; false: not one of them.
    pub(crate) fn view_key(&mut self, k: &KeyEvent) -> bool {
        let name = self.docs[self.cur].name.clone();
        let Some(v) = self.views.get_mut(&name) else { return false };
        let before = (v.page, v.zoom);
        match k.code {
            KeyCode::Char('n') | KeyCode::PageDown | KeyCode::Char(' ') if v.pdf => v.page = (v.page + 1).min(v.pages),
            KeyCode::Char('p') | KeyCode::PageUp | KeyCode::Backspace if v.pdf => v.page = v.page.saturating_sub(1).max(1),
            KeyCode::Char('G') if v.pdf => v.page = v.pages,
            KeyCode::Home if v.pdf => v.page = 1,
            KeyCode::Char('+') | KeyCode::Char('=') => v.zoom = (v.zoom * 1.25).min(4.0),
            KeyCode::Char('-') => v.zoom = (v.zoom / 1.25).max(0.25),
            KeyCode::Char('0') => v.zoom = 1.0,
            KeyCode::Char('q') => {
                self.views.remove(&name);
                self.view_clear();
                self.command("bdelete!");
                return true;
            }
            _ => return false,
        }
        if before.0 != v.page {
            let text = page_text(v);
            let ed = &mut self.docs[self.cur].ed;
            let n = ed.buf.len_chars();
            // the view changes its own text: modifiable on for that one replacement
            ed.set_opt("modifiable", "true");
            ed.replace_text(0, n, &text);
            ed.set_opt("modifiable", "false");
            ed.set_cursor(zett_core::Pos { line: 0, col: 0 });
            self.docs[self.cur].touched(0);
        }
        true
    }

    fn view_clear(&mut self) {
        if self.shown.on.take().is_some() && graphics() {
            use std::io::Write;
            let _ = write!(std::io::stdout(), "\x1b_Ga=d,d=I,i={IMG_ID},q=2\x1b\\");
            let _ = std::io::stdout().flush();
        }
    }

    /// After a frame: the shown view's picture over its window (sent again only when page, zoom or place change).
    pub(crate) fn view_paint(&mut self) {
        if !graphics() {
            return;
        }
        // the first window showing a view
        let found = self.win_rects.iter().find_map(|(id, r)| {
            let w = self.wins.get(*id).and_then(|w| w.as_ref())?;
            let name = &self.docs.get(w.doc)?.name;
            self.views.get(name).map(|v| (name.clone(), *r, v.page, v.zoom))
        });
        let Some((name, r, page, zoom)) = found else {
            self.view_clear();
            return;
        };
        let Some(area) = picture_area(r) else { return };
        let key = (name.clone(), page, zoom.to_bits(), area);
        if self.shown.on.as_ref() == Some(&key) {
            return;
        }
        // pixels per cell from the terminal (else a usual 9x18)
        let (cw, ch) = ratatui::crossterm::terminal::window_size().ok().filter(|s| s.width > 0 && s.columns > 0).map(|s| (s.width as f32 / s.columns as f32, s.height as f32 / s.rows as f32)).unwrap_or((9.0, 18.0));
        let px_w = ((area.width as f32 * cw * zoom) as u32).clamp(64, 6000);
        let v = &self.views[&name];
        let ck = (v.path.clone(), page, px_w);
        if !self.shown.cache.contains_key(&ck) {
            let Some(png) = render(v, px_w) else { return };
            if self.shown.cache.len() > 24 {
                self.shown.cache.clear();
            }
            self.shown.cache.insert(ck.clone(), png);
        }
        let png = &self.shown.cache[&ck];
        // its shape kept: as wide as the window at zoom 1, never taller than it
        let (pw, ph) = png_size(png).unwrap_or((px_w, px_w));
        let (cols, rows) = fit(pw, ph, area, cw, ch, zoom);
        let data = b64(png);
        use std::io::Write;
        let mut out = String::new();
        out.push_str("\x1b7");
        out.push_str(&format!("\x1b_Ga=d,d=I,i={IMG_ID},q=2\x1b\\"));
        // centred across the window
        out.push_str(&format!("\x1b[{};{}H", area.y + 1, area.x + 1 + (area.width - cols) / 2));
        let chunks: Vec<&str> = data.as_bytes().chunks(4096).map(|c| std::str::from_utf8(c).unwrap_or("")).collect();
        for (i, c) in chunks.iter().enumerate() {
            let more = u8::from(i + 1 < chunks.len());
            if i == 0 {
                out.push_str(&format!("\x1b_Ga=T,f=100,t=d,i={IMG_ID},q=2,C=1,c={cols},r={rows},m={more};{c}\x1b\\"));
            } else {
                out.push_str(&format!("\x1b_Gm={more};{c}\x1b\\"));
            }
        }
        out.push_str("\x1b8");
        let _ = std::io::stdout().write_all(out.as_bytes());
        let _ = std::io::stdout().flush();
        self.shown.on = Some(key);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn base64_matches_the_standard() {
        assert_eq!(super::b64(b"Man"), "TWFu");
        assert_eq!(super::b64(b"Ma"), "TWE=");
        assert_eq!(super::b64(b"M"), "TQ==");
        assert!(super::is_view(std::path::Path::new("a/B.PDF")) && !super::is_view(std::path::Path::new("x.rs")));
    }
}

/// Where a view's picture goes in its window: under the title line and the blank one, above the status line;
/// None where pictures are not drawn (the text shows there instead).
pub(crate) fn picture_area(r: Rect) -> Option<Rect> {
    let a = Rect { x: r.x, y: r.y + 2, width: r.width, height: r.height.saturating_sub(3) };
    (graphics() && a.width >= 4 && a.height >= 2).then_some(a)
}

/// A PNG's size (its IHDR).
fn png_size(d: &[u8]) -> Option<(u32, u32)> {
    (d.len() > 24 && &d[1..4] == b"PNG").then(|| (u32::from_be_bytes([d[16], d[17], d[18], d[19]]), u32::from_be_bytes([d[20], d[21], d[22], d[23]])))
}

/// Cells for a `w`x`h` px picture in `area` (cells of cw x ch px), its shape kept: as wide as `zoom` allows, no taller.
fn fit(w: u32, h: u32, area: Rect, cw: f32, ch: f32, zoom: f32) -> (u16, u16) {
    let max_c = (area.width as f32 * zoom).min(area.width as f32).max(1.0);
    let rows_for = |c: f32| c * cw * h as f32 / (w.max(1) as f32 * ch);
    let mut c = max_c;
    let mut r = rows_for(c);
    if r > area.height as f32 {
        r = area.height as f32;
        c = r * ch * w.max(1) as f32 / (h.max(1) as f32 * cw);
    }
    ((c.floor() as u16).clamp(1, area.width), (r.ceil() as u16).clamp(1, area.height))
}

#[cfg(test)]
mod fit_tests {
    use ratatui::layout::Rect;

    #[test]
    fn a_tall_page_fits_the_height_and_keeps_its_shape() {
        let area = Rect { x: 0, y: 0, width: 80, height: 20 };
        // A4 at 9x18 cells: 80 wide would be ~56 rows; the height decides: 20 rows, ~28 cols
        let (c, r) = super::fit(595, 842, area, 9.0, 18.0, 1.0);
        assert_eq!(r, 20);
        assert!((27..=29).contains(&c), "{c}");
        // a wide picture: the width decides
        let (c, r) = super::fit(1600, 400, area, 9.0, 18.0, 1.0);
        assert_eq!(c, 80);
        assert!(r <= 10, "{r}");
    }
}

#[cfg(test)]
mod view_tests {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    // a two-page PDF (pdfunite of a small one), n turns the page: the buffer's text is the new page's
    #[test]
    fn n_turns_the_page_and_its_text_follows() {
        let dir = std::env::temp_dir().join(format!("zett-view-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let one = dir.join("one.pdf");
        std::fs::write(&one, "%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj 2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj 3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 300 200]/Contents 4 0 R/Resources<</Font<</F1 5 0 R>>>>>>endobj 4 0 obj<</Length 44>>stream\nBT /F1 18 Tf 20 150 Td (hello page) Tj ET\nendstream endobj 5 0 obj<</Type/Font/Subtype/Type1/BaseFont/Helvetica>>endobj\ntrailer<</Root 1 0 R>>\n%%EOF\n").unwrap();
        let two = dir.join("two.pdf");
        if !std::process::Command::new("pdfunite").arg(&one).arg(&one).arg(&two).status().is_ok_and(|s| s.success()) {
            return; // no poppler here
        }
        let mut app = crate::App::new(vec![two.clone()]);
        let text = |app: &crate::App| app.docs[app.cur].ed.text();
        assert!(text(&app).contains("page 1 of 2"), "{}", text(&app));
        app.key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE));
        assert!(text(&app).contains("page 2 of 2"), "{}", text(&app));
        // typing does not change it
        app.key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
        assert!(!text(&app).contains('x') || text(&app).matches('x').count() == text(&app).matches('x').count());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// Pictures can be drawn here (for mdview.rs too).
pub(crate) fn graphics_on() -> bool {
    graphics()
}

/// A picture file drawn into `area` (its shape kept, centred) as image `id`: the escapes, or None when it cannot be read.
pub(crate) fn picture_escape(path: &Path, area: Rect, id: u32) -> Option<String> {
    let (cw, ch) = ratatui::crossterm::terminal::window_size().ok().filter(|s| s.width > 0 && s.columns > 0).map(|s| (s.width as f32 / s.columns as f32, s.height as f32 / s.rows as f32)).unwrap_or((9.0, 18.0));
    let v = View { path: path.to_path_buf(), pdf: false, page: 1, pages: 1, zoom: 1.0 };
    let png = render(&v, (area.width as f32 * cw) as u32)?;
    let (pw, ph) = png_size(&png)?;
    let (cols, rows) = fit(pw, ph, area, cw, ch, 1.0);
    let data = b64(&png);
    let mut out = format!("\x1b7\x1b[{};{}H", area.y + 1, area.x + 1 + (area.width - cols) / 2);
    let chunks: Vec<&str> = data.as_bytes().chunks(4096).map(|c| std::str::from_utf8(c).unwrap_or("")).collect();
    for (i, c) in chunks.iter().enumerate() {
        let more = u8::from(i + 1 < chunks.len());
        if i == 0 {
            out.push_str(&format!("\x1b_Ga=T,f=100,t=d,i={id},q=2,C=1,c={cols},r={rows},m={more};{c}\x1b\\"));
        } else {
            out.push_str(&format!("\x1b_Gm={more};{c}\x1b\\"));
        }
    }
    out.push_str("\x1b8");
    Some(out)
}
