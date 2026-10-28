//! The workspace: one tree of blocks in a Loro document (a CRDT).
//!
//! Every block is a tree node. Its meta map holds `type` and the type's fields
//! (`level`, `checked`, `lang`) and a `text` container with its rich text.
//! A page is a block whose children are its content. Because it all lives in
//! one CRDT, sync, history and branches come with it, not bolted on later.
//!
//! No file system here (WASM-clean): saving is `export()` bytes, loading is
//! `import()`.

use loro::{ExportMode, LoroDoc, LoroMap, LoroText, LoroTree, TreeID, TreeParentId, ValueOrContainer};

pub type BlockId = TreeID;

/// What a block is. Its text lives beside it, in the CRDT.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Page,
    Paragraph,
    Heading(u8),
    Todo { checked: bool },
    Bullet,
    Quote,
    Code { lang: String },
}

impl Kind {
    fn name(&self) -> &'static str {
        match self {
            Kind::Page => "page",
            Kind::Paragraph => "paragraph",
            Kind::Heading(_) => "heading",
            Kind::Todo { .. } => "todo",
            Kind::Bullet => "bullet",
            Kind::Quote => "quote",
            Kind::Code { .. } => "code",
        }
    }
}

#[derive(Debug)]
pub struct Error(pub String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
fn err(e: impl std::fmt::Display) -> Error {
    Error(e.to_string())
}
pub type Result<T> = std::result::Result<T, Error>;

pub struct Workspace {
    doc: LoroDoc,
    tree: LoroTree,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    pub fn new() -> Self {
        let doc = LoroDoc::new();
        let tree = doc.get_tree("blocks");
        // ordered children: blocks have positions, not only parents
        tree.enable_fractional_index(0);
        Self { doc, tree }
    }

    /// A workspace from bytes `export()` made (a snapshot or updates).
    pub fn import(bytes: &[u8]) -> Result<Self> {
        let ws = Self::new();
        ws.doc.import(bytes).map_err(err)?;
        Ok(ws)
    }

    /// Ends the current change: one step for undo (the window calls it after every edit).
    pub fn commit(&self) {
        self.doc.commit();
    }

    /// Undo / redo over this workspace's own changes (edits within `merge_ms` are one step).
    pub fn undo_manager(&self, merge_ms: i64) -> loro::UndoManager {
        let mut u = loro::UndoManager::new(&self.doc);
        u.set_merge_interval(merge_ms);
        u.set_max_undo_steps(500);
        u
    }

    /// The whole workspace as bytes, history included.
    pub fn export(&self) -> Result<Vec<u8>> {
        self.doc.commit();
        self.doc.export(ExportMode::Snapshot).map_err(err)
    }

    /// Merges another copy's changes into this one (sync is this, repeated).
    pub fn merge(&self, other: &Workspace) -> Result<()> {
        let bytes = other.export()?;
        self.doc.import(&bytes).map_err(err)?;
        Ok(())
    }

    // ── structure ──

    /// Top-level pages, in order.
    pub fn pages(&self) -> Vec<BlockId> {
        self.tree.children(TreeParentId::Root).unwrap_or_default()
    }

    pub fn children(&self, id: BlockId) -> Vec<BlockId> {
        self.tree.children(id).unwrap_or_default()
    }

    pub fn parent(&self, id: BlockId) -> Option<BlockId> {
        match self.tree.parent(id) {
            Some(TreeParentId::Node(p)) => Some(p),
            _ => None,
        }
    }

    pub fn new_page(&self, title: &str) -> Result<BlockId> {
        let id = self.tree.create(TreeParentId::Root).map_err(err)?;
        self.init(id, &Kind::Page, title)?;
        Ok(id)
    }

    /// A new block as the last child of `parent`.
    pub fn append(&self, parent: BlockId, kind: Kind, text: &str) -> Result<BlockId> {
        let id = self.tree.create(parent).map_err(err)?;
        self.init(id, &kind, text)?;
        Ok(id)
    }

    /// A new block at `index` among `parent`'s children.
    pub fn insert(&self, parent: BlockId, index: usize, kind: Kind, text: &str) -> Result<BlockId> {
        let id = self.tree.create_at(parent, index).map_err(err)?;
        self.init(id, &kind, text)?;
        Ok(id)
    }

    pub fn move_to(&self, id: BlockId, parent: BlockId, index: usize) -> Result<()> {
        self.tree.mov_to(id, parent, index).map_err(err)
    }

    pub fn remove(&self, id: BlockId) -> Result<()> {
        self.tree.delete(id).map_err(err)
    }

    pub fn exists(&self, id: BlockId) -> bool {
        self.tree.contains(id) && !self.tree.is_node_deleted(&id).unwrap_or(true)
    }

    fn init(&self, id: BlockId, kind: &Kind, text: &str) -> Result<()> {
        self.set_kind(id, kind)?;
        // a new block's text is empty: insert, no diff needed
        if text.is_empty() { Ok(()) } else { self.insert_text(id, 0, text) }
    }

    fn meta(&self, id: BlockId) -> Result<LoroMap> {
        self.tree.get_meta(id).map_err(err)
    }

    // ── content ──

    pub fn kind(&self, id: BlockId) -> Kind {
        let Ok(m) = self.meta(id) else { return Kind::Paragraph };
        let s = |k: &str| get_str(&m, k);
        match s("type").as_deref() {
            Some("page") => Kind::Page,
            Some("heading") => Kind::Heading(get_i64(&m, "level").unwrap_or(1).clamp(1, 6) as u8),
            Some("todo") => Kind::Todo { checked: get_bool(&m, "checked").unwrap_or(false) },
            Some("bullet") => Kind::Bullet,
            Some("quote") => Kind::Quote,
            Some("code") => Kind::Code { lang: s("lang").unwrap_or_default() },
            _ => Kind::Paragraph,
        }
    }

    pub fn set_kind(&self, id: BlockId, kind: &Kind) -> Result<()> {
        let m = self.meta(id)?;
        m.insert("type", kind.name()).map_err(err)?;
        match kind {
            Kind::Heading(l) => m.insert("level", *l as i64).map_err(err)?,
            Kind::Todo { checked } => m.insert("checked", *checked).map_err(err)?,
            Kind::Code { lang } => m.insert("lang", lang.as_str()).map_err(err)?,
            _ => {}
        }
        Ok(())
    }

    fn text_of(&self, id: BlockId) -> Result<LoroText> {
        // mergeable: two devices that both create a block's text get ONE text, not two
        self.meta(id)?.ensure_mergeable_text("text").map_err(err)
    }

    pub fn text(&self, id: BlockId) -> String {
        self.text_of(id).map(|t| t.to_string()).unwrap_or_default()
    }

    /// Replaces a block's text with a diff, so concurrent edits elsewhere in it survive.
    pub fn set_text(&self, id: BlockId, text: &str) -> Result<()> {
        self.text_of(id)?.update(text, Default::default()).map_err(err)
    }

    /// Inserts at a char position of the block's text.
    pub fn insert_text(&self, id: BlockId, at: usize, s: &str) -> Result<()> {
        self.text_of(id)?.insert(at, s).map_err(err)
    }

    /// Deletes `len` chars from `at`.
    pub fn delete_text(&self, id: BlockId, at: usize, len: usize) -> Result<()> {
        self.text_of(id)?.delete(at, len).map_err(err)
    }

    /// A page by its title (first match), for links and the CLI.
    pub fn find_page(&self, title: &str) -> Option<BlockId> {
        self.pages().into_iter().find(|p| self.text(*p) == title)
    }
}

fn get_value(m: &LoroMap, k: &str) -> Option<loro::LoroValue> {
    match m.get(k)? {
        ValueOrContainer::Value(v) => Some(v),
        _ => None,
    }
}
fn get_str(m: &LoroMap, k: &str) -> Option<String> {
    get_value(m, k)?.into_string().ok().map(|s| s.to_string())
}
fn get_i64(m: &LoroMap, k: &str) -> Option<i64> {
    get_value(m, k)?.into_i64().ok()
}
fn get_bool(m: &LoroMap, k: &str) -> Option<bool> {
    get_value(m, k)?.into_bool().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_and_blocks() {
        let ws = Workspace::new();
        let p = ws.new_page("Plans").unwrap();
        let a = ws.append(p, Kind::Heading(2), "Goals").unwrap();
        let b = ws.append(p, Kind::Todo { checked: false }, "ship M0").unwrap();
        assert_eq!(ws.pages(), vec![p]);
        assert_eq!(ws.children(p), vec![a, b]);
        assert_eq!(ws.kind(a), Kind::Heading(2));
        assert_eq!(ws.text(b), "ship M0");
        assert_eq!(ws.parent(b), Some(p));
        assert_eq!(ws.find_page("Plans"), Some(p));
    }

    #[test]
    fn move_and_remove() {
        let ws = Workspace::new();
        let p = ws.new_page("p").unwrap();
        let a = ws.append(p, Kind::Paragraph, "a").unwrap();
        let b = ws.append(p, Kind::Paragraph, "b").unwrap();
        ws.move_to(b, p, 0).unwrap();
        assert_eq!(ws.children(p), vec![b, a]);
        ws.remove(a).unwrap();
        assert_eq!(ws.children(p), vec![b]);
        assert!(!ws.exists(a));
    }

    #[test]
    fn text_edits() {
        let ws = Workspace::new();
        let p = ws.new_page("p").unwrap();
        let a = ws.append(p, Kind::Paragraph, "hello world").unwrap();
        ws.insert_text(a, 5, ",").unwrap();
        assert_eq!(ws.text(a), "hello, world");
        ws.delete_text(a, 5, 1).unwrap();
        ws.set_text(a, "hello there world").unwrap();
        assert_eq!(ws.text(a), "hello there world");
    }

    #[test]
    fn export_import_roundtrip() {
        let ws = Workspace::new();
        let p = ws.new_page("Saved").unwrap();
        ws.append(p, Kind::Code { lang: "rust".into() }, "fn main() {}").unwrap();
        let back = Workspace::import(&ws.export().unwrap()).unwrap();
        let p2 = back.find_page("Saved").unwrap();
        let c = back.children(p2)[0];
        assert_eq!(back.kind(c), Kind::Code { lang: "rust".into() });
        assert_eq!(back.text(c), "fn main() {}");
    }

    /// The point of the CRDT: two devices edit offline, then merge, and both edits survive.
    #[test]
    fn concurrent_edits_merge() {
        let laptop = Workspace::new();
        let p = laptop.new_page("Shared").unwrap();
        let a = laptop.append(p, Kind::Paragraph, "the cat sat").unwrap();
        let phone = Workspace::import(&laptop.export().unwrap()).unwrap();

        laptop.insert_text(a, 4, "black ").unwrap();      // "the black cat sat"
        phone.insert_text(a, 11, " down").unwrap();        // "the cat sat down"
        phone.append(p, Kind::Todo { checked: false }, "feed it").unwrap();

        laptop.merge(&phone).unwrap();
        phone.merge(&laptop).unwrap();
        assert_eq!(laptop.text(a), "the black cat sat down");
        assert_eq!(phone.text(a), laptop.text(a));
        assert_eq!(laptop.children(p).len(), 2);
    }
}
