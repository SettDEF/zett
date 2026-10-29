//! WASM plugins (zett.wasm): a module sandboxed by wasmi, text in and text out. The module exports its `memory`,
//! `alloc(len) -> ptr`, and functions `name(ptr, len) -> i64` (the answer's ptr << 32 | len, UTF-8). It may import
//! `env.zett_log(ptr, len)`. It reaches no file, process or network; each call runs on a fuel budget, so a loop
//! that never ends is stopped instead of freezing the editor.
use mlua::{Lua, Table, UserData, UserDataMethods};
use std::cell::RefCell;
use wasmi::{Caller, Engine, Extern, Instance, Linker, Memory, Module, Store};

/// Instructions one call may run (about a second's work on this laptop).
const FUEL: u64 = 200_000_000;

struct Plugin {
    store: RefCell<Store<Vec<String>>>,
    instance: Instance,
    memory: Memory,
}

fn read(mem: &Memory, store: &Store<Vec<String>>, ptr: u32, len: u32) -> Result<String, String> {
    let data = mem.data(store);
    let (a, b) = (ptr as usize, ptr as usize + len as usize);
    data.get(a..b).map(|s| String::from_utf8_lossy(s).into_owned()).ok_or_else(|| "the answer lies outside its memory".into())
}

impl Plugin {
    fn load(bytes: &[u8]) -> Result<Plugin, String> {
        let mut config = wasmi::Config::default();
        config.consume_fuel(true);
        let engine = Engine::new(&config);
        let module = Module::new(&engine, bytes).map_err(|e| e.to_string())?;
        let mut store = Store::new(&engine, Vec::new());
        let mut linker = <Linker<Vec<String>>>::new(&engine);
        linker
            .func_wrap("env", "zett_log", |mut caller: Caller<'_, Vec<String>>, ptr: i32, len: i32| {
                if let Some(Extern::Memory(m)) = caller.get_export("memory") {
                    let s = { let d = m.data(&caller); d.get(ptr as usize..(ptr + len) as usize).map(|b| String::from_utf8_lossy(b).into_owned()) };
                    if let Some(s) = s { caller.data_mut().push(s) }
                }
            })
            .map_err(|e| e.to_string())?;
        store.set_fuel(FUEL).map_err(|e| e.to_string())?;
        let instance = linker.instantiate_and_start(&mut store, &module).map_err(|e| e.to_string())?;
        let memory = instance.get_memory(&store, "memory").ok_or("the module exports no memory")?;
        Ok(Plugin { store: RefCell::new(store), instance, memory })
    }

    /// `name(input)`: the text it gives back.
    fn call(&self, name: &str, input: &str) -> Result<String, String> {
        let mut store = self.store.borrow_mut();
        store.set_fuel(FUEL).map_err(|e| e.to_string())?;
        let alloc = self.instance.get_typed_func::<i32, i32>(&*store, "alloc").map_err(|_| "the module exports no alloc(len) -> ptr")?;
        let f = self.instance.get_typed_func::<(i32, i32), i64>(&*store, name).map_err(|_| format!("no function {name}(ptr, len) -> i64"))?;
        let ptr = alloc.call(&mut *store, input.len() as i32).map_err(|e| e.to_string())?;
        self.memory.write(&mut *store, ptr as usize, input.as_bytes()).map_err(|e| e.to_string())?;
        let out = f.call(&mut *store, (ptr, input.len() as i32)).map_err(|e| {
            let s = e.to_string();
            if s.contains("fuel") { format!("{name}: stopped, it ran too long") } else { s }
        })? as u64;
        read(&self.memory, &store, (out >> 32) as u32, out as u32)
    }

    fn exports(&self) -> Vec<String> {
        let store = self.store.borrow();
        self.instance.exports(&*store).filter(|e| e.clone().into_func().is_some()).map(|e| e.name().to_string()).filter(|n| n != "alloc").collect()
    }
}

impl UserData for Plugin {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_method("call", |_, p, (name, input): (String, Option<String>)| p.call(&name, &input.unwrap_or_default()).map_err(mlua::Error::runtime));
        m.add_method("exports", |_, p, ()| Ok(p.exports()));
        // what the module logged with zett_log since the last look
        m.add_method("logs", |_, p, ()| Ok(std::mem::take(p.store.borrow_mut().data_mut())));
    }
}

/// zett.wasm.load(path) → a plugin: p:call(name, text), p:exports(), p:logs()
pub(crate) fn register(lua: &Lua, z: &Table) -> mlua::Result<()> {
    let w = lua.create_table()?;
    w.set("load", lua.create_function(|_, path: String| {
        let bytes = std::fs::read(&path).map_err(|e| mlua::Error::runtime(format!("{path}: {e}")))?;
        Plugin::load(&bytes).map_err(|e| mlua::Error::runtime(format!("{path}: {e}")))
    })?)?;
    z.set("wasm", w)
}

#[cfg(test)]
mod tests {
    // a module in text form: upper-cases ASCII; another that never stops
    const UPPER: &str = r#"(module
      (import "env" "zett_log" (func $log (param i32 i32)))
      (memory (export "memory") 1)
      (global $top (mut i32) (i32.const 1024))
      (func (export "alloc") (param $n i32) (result i32) (local $p i32)
        (local.set $p (global.get $top)) (global.set $top (i32.add (global.get $top) (local.get $n))) (local.get $p))
      (func (export "upper") (param $p i32) (param $n i32) (result i64) (local $i i32) (local $c i32)
        (call $log (local.get $p) (local.get $n))
        (block $done (loop $next
          (br_if $done (i32.ge_u (local.get $i) (local.get $n)))
          (local.set $c (i32.load8_u (i32.add (local.get $p) (local.get $i))))
          (if (i32.and (i32.ge_u (local.get $c) (i32.const 97)) (i32.le_u (local.get $c) (i32.const 122)))
            (then (i32.store8 (i32.add (local.get $p) (local.get $i)) (i32.sub (local.get $c) (i32.const 32)))))
          (local.set $i (i32.add (local.get $i) (i32.const 1))) (br $next)))
        (i64.or (i64.shl (i64.extend_i32_u (local.get $p)) (i64.const 32)) (i64.extend_i32_u (local.get $n))))
      (func (export "forever") (param i32 i32) (result i64) (loop $l (br $l)) (i64.const 0)))"#;

    #[test]
    fn a_wasm_plugin_maps_text_and_cannot_hang() {
        let bytes = wat::parse_str(UPPER).unwrap();
        let p = super::Plugin::load(&bytes).unwrap();
        assert_eq!(p.call("upper", "hello zett").unwrap(), "HELLO ZETT");
        assert_eq!(p.store.borrow().data(), &vec!["hello zett".to_string()], "zett_log reached the host");
        let e = p.call("forever", "").unwrap_err();
        assert!(e.contains("ran too long"), "{e}");
        assert!(p.exports().contains(&"upper".to_string()));
    }
}
