#!/bin/sh
# gen-charnames.sh — writes zett-tui's character names (C-x 8 RET) and emoji groups (C-x 8 e) from
# the Emacs installed here: (ucs-names) without the big name-by-number blocks (CJK ideographs,
# Hangul syllables, Tangut …), and emoji-labels.el's groups.
# Usage: scripts/gen-charnames.sh        (needs emacs)
set -eu
cd "$(dirname "$0")/.."
EMACS=${EMACS:-/usr/bin/emacs}
command -v "$EMACS" >/dev/null || { echo "gen-charnames: needs emacs" >&2; exit 1; }
dir=crates/zett-tui/src/data
mkdir -p "$dir"
"$EMACS" -Q --batch --eval "
(let ((skip \"\\\\\`\\\\(CJK \\\\(UNIFIED\\\\|COMPATIBILITY\\\\) IDEOGRAPH\\\\|HANGUL SYLLABLE\\\\|TANGUT\\\\|KHITAN\\\\|NUSHU\\\\|EGYPTIAN HIEROGLYPH\\\\|CUNEIFORM\\\\|ANATOLIAN\\\\|BAMUM\\\\|LINEAR [AB]\\\\|YI SYLLABLE\\\\|VARIATION SELECTOR\\\\|TAG \\\\)\")
      rows)
  (maphash (lambda (k v) (unless (string-match-p skip k) (push (format \"%s\t%x\" k v) rows))) (ucs-names))
  (with-temp-file \"$dir/charnames.txt\"
    (insert (mapconcat #'identity (sort rows #'string<) \"\n\") \"\n\"))
  (load \"emoji-labels\" nil t)
  (with-temp-file \"$dir/emoji.txt\"
    (letrec ((walk (lambda (path node)
                     (let ((here (seq-filter #'stringp (cdr node))))
                       (when here (insert path \"\t\" (mapconcat #'identity here \" \") \"\n\"))
                       (dolist (c (cdr node))
                         (when (consp c) (funcall walk (concat path \" / \" (car c)) c)))))))
      (dolist (g emoji--labels) (funcall walk (car g) g)))))"
echo "gen-charnames: $(wc -l < "$dir/charnames.txt") names, $(wc -l < "$dir/emoji.txt") emoji groups in $dir"
