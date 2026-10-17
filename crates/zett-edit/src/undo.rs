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
