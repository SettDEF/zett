//! text-tools: a zett WASM plugin. Each export takes the text of the lines (ptr, len) and gives the new text back
//! as ptr << 32 | len (zett-script/src/wasm.rs has the whole contract).

#[no_mangle]
pub extern "C" fn alloc(len: i32) -> i32 {
    let mut v = Vec::<u8>::with_capacity(len.max(0) as usize);
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p as i32
}

// given by zett (the host)
#[link(wasm_import_module = "env")]
extern "C" {
    fn zett_log(ptr: i32, len: i32);
}

fn input(ptr: i32, len: i32) -> String {
    let s = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
    String::from_utf8_lossy(s).into_owned()
}

fn answer(s: String) -> i64 {
    let b = s.into_bytes().into_boxed_slice();
    let (p, n) = (b.as_ptr() as i64, b.len() as i64);
    std::mem::forget(b);
    (p << 32) | n
}

fn log(s: &str) {
    unsafe { zett_log(s.as_ptr() as i32, s.len() as i32) }
}

#[no_mangle]
pub extern "C" fn sort_lines(ptr: i32, len: i32) -> i64 {
    let t = input(ptr, len);
    let mut l: Vec<&str> = t.split('\n').collect();
    l.sort_by_key(|s| s.to_lowercase());
    answer(l.join("\n"))
}

#[no_mangle]
pub extern "C" fn unique_lines(ptr: i32, len: i32) -> i64 {
    let t = input(ptr, len);
    let mut seen = std::collections::HashSet::new();
    let out: Vec<&str> = t.split('\n').filter(|l| seen.insert(*l)).collect();
    let gone = t.split('\n').count() - out.len();
    if gone > 0 {
        log(&format!("{gone} repeated line{} taken out", if gone == 1 { "" } else { "s" }));
    }
    answer(out.join("\n"))
}

#[no_mangle]
pub extern "C" fn reverse_lines(ptr: i32, len: i32) -> i64 {
    let t = input(ptr, len);
    answer(t.split('\n').rev().collect::<Vec<_>>().join("\n"))
}

#[no_mangle]
pub extern "C" fn title_case(ptr: i32, len: i32) -> i64 {
    let t = input(ptr, len);
    let mut out = String::with_capacity(t.len());
    let mut start = true;
    for c in t.chars() {
        if start && c.is_alphabetic() { out.extend(c.to_uppercase()) } else { out.push(c) }
        start = !c.is_alphanumeric() && c != '\'';
    }
    answer(out)
}

#[no_mangle]
pub extern "C" fn json_pretty(ptr: i32, len: i32) -> i64 {
    let t = input(ptr, len);
    match serde_json::from_str::<serde_json::Value>(&t) {
        Ok(v) => answer(serde_json::to_string_pretty(&v).unwrap_or(t)),
        Err(e) => {
            log(&format!("not JSON: {e}"));
            answer(t)
        }
    }
}
