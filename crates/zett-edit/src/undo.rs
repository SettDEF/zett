//! Undo as a tree, as Vim keeps it: undoing and then changing starts a branch, nothing is lost.

use zett_core::Buffer;

#[derive(Clone, Debug)]
pub(crate) struct Edit {
    pub at: usize,
    pub removed: String,
    pub inserted: String,
}

/// One undo step: the edits of one change (a whole insert session is one).
/// Only what changed is kept, never a copy of the text: undo costs the size
/// of the change, not the size of the file.
#[derive(Clone, Debug, Default)]
pub(crate) struct Change {
    pub edits: Vec<Edit>,
    pub cursor: usize,
}

impl Change {
    pub fn revert(&self, buf: &mut Buffer) {
        for e in self.edits.iter().rev() {
            buf.remove(e.at, e.at + e.inserted.chars().count());
            buf.insert(e.at, &e.removed);
        }
    }
    pub fn apply(&self, buf: &mut Buffer) {
        for e in &self.edits {
            buf.remove(e.at, e.at + e.removed.chars().count());
            buf.insert(e.at, &e.inserted);
        }
    }
}

#[derive(Debug)]
pub(crate) struct Node {
    pub parent: usize,
    pub change: Change,
    /// seconds since the epoch (0 where there is no clock)
    pub time: u64,
    /// the child `C-r` goes back into: the one last undone or made
    pub redo: Option<usize>,
}

/// Node 0 is the text as opened; a node's number is its place in time (Vim's seq).
#[derive(Debug)]
pub(crate) struct Tree {
    pub nodes: Vec<Node>,
    pub cur: usize,
}

fn now() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    return std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    #[cfg(target_arch = "wasm32")]
    0
}

impl Default for Tree {
    fn default() -> Self {
        Tree { nodes: vec![Node { parent: 0, change: Change::default(), time: now(), redo: None }], cur: 0 }
    }
}

impl Tree {
    pub fn push(&mut self, change: Change) {
        let n = self.nodes.len();
        self.nodes.push(Node { parent: self.cur, change, time: now(), redo: None });
        self.nodes[self.cur].redo = Some(n);
        self.cur = n;
    }

    /// The way from `from` to `to`: the nodes to revert (going up), then to apply (going down).
    pub fn path(&self, from: usize, to: usize) -> (Vec<usize>, Vec<usize>) {
        let chain = |mut n: usize| {
            let mut v = vec![n];
            while n != 0 {
                n = self.nodes[n].parent;
                v.push(n);
            }
            v
        };
        let (a, b) = (chain(from), chain(to));
        let common = *a.iter().find(|n| b.contains(n)).unwrap_or(&0);
        let up = a.into_iter().take_while(|&n| n != common).collect();
        let mut down: Vec<usize> = b.into_iter().take_while(|&n| n != common).collect();
        down.reverse();
        (up, down)
    }

    /// The tree as bytes, for the text `text` it is at now (its fingerprint is kept, so a file
    /// changed elsewhere is not given a history that does not fit it).
    pub fn save(&self, text: &str) -> Vec<u8> {
        let mut out = format!("zett-undo 1\n{} {} {}\n", self.cur, fingerprint(text), self.nodes.len()).into_bytes();
        for n in &self.nodes {
            let redo = n.redo.map_or(-1, |r| r as i64);
            out.extend(format!("{} {} {} {} {}\n", n.parent, n.time, redo, n.change.cursor, n.change.edits.len()).bytes());
            for e in &n.change.edits {
                out.extend(format!("{} {} {}\n", e.at, e.removed.len(), e.inserted.len()).bytes());
                out.extend(e.removed.bytes());
                out.extend(e.inserted.bytes());
            }
        }
        out
    }

    /// A saved tree, if it was saved for exactly this text.
    pub fn load(bytes: &[u8], text: &str) -> Option<Tree> {
        let mut r = Reader { b: bytes, at: 0 };
        if r.line()? != "zett-undo 1" {
            return None;
        }
        let head = r.line()?;
        let mut h = head.split(' ');
        let cur: usize = h.next()?.parse().ok()?;
        let fp: u64 = h.next()?.parse().ok()?;
        let count: usize = h.next()?.parse().ok()?;
        if fp != fingerprint(text) || cur >= count {
            return None;
        }
        let mut nodes = Vec::with_capacity(count);
        for _ in 0..count {
            let l = r.line()?;
            let v: Vec<i64> = l.split(' ').map(|x| x.parse().ok()).collect::<Option<_>>()?;
            let [parent, time, redo, cursor, edits] = v[..] else { return None };
            let mut change = Change { edits: Vec::with_capacity(edits as usize), cursor: cursor as usize };
            for _ in 0..edits {
                let l = r.line()?;
                let v: Vec<usize> = l.split(' ').map(|x| x.parse().ok()).collect::<Option<_>>()?;
                let [at, rl, il] = v[..] else { return None };
                let removed = r.take(rl)?;
                let inserted = r.take(il)?;
                change.edits.push(Edit { at, removed, inserted });
            }
            if parent as usize >= count {
                return None;
            }
            nodes.push(Node { parent: parent as usize, change, time: time as u64, redo: (redo >= 0).then_some(redo as usize) });
        }
        Some(Tree { nodes, cur })
    }

    /// The text at node `to`, worked out from `text` at the current node.
    pub fn text_at(&self, text: &str, to: usize) -> String {
        let mut buf = Buffer::from_str(text);
        let (up, down) = self.path(self.cur, to);
        for n in up {
            self.nodes[n].change.revert(&mut buf);
        }
        for n in down {
            self.nodes[n].change.apply(&mut buf);
        }
        buf.text()
    }
}

/// FNV-1a over the text: which text a saved history belongs to.
pub(crate) fn fingerprint(text: &str) -> u64 {
    text.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3))
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn line(&mut self) -> Option<String> {
        let end = self.b[self.at..].iter().position(|&c| c == b'\n')? + self.at;
        let s = std::str::from_utf8(&self.b[self.at..end]).ok()?.to_string();
        self.at = end + 1;
        Some(s)
    }
    fn take(&mut self, n: usize) -> Option<String> {
        let s = std::str::from_utf8(self.b.get(self.at..self.at + n)?).ok()?.to_string();
        self.at += n;
        Some(s)
    }
}
