# Emacs's commands

Generated 2026-10-21 by scripts/parity.sh from NVIM v0.12.6, GNU Emacs 31.1 and zett 0.1.0.

All 3163 interactive commands of a plain Emacs (`emacs -Q`), by their package's prefix. ✓ zett has it under
the same name. Most are packages (mail, games, modes for other languages): zett's answer to many is a
plugin, not the core.


## 2C

- · `2C-associate-buffer` Associate another BUFFER with this one in two-column minor mode.
- · `2C-command` 
- · `2C-split` Split a two-column text at point, into two buffers in two-column minor mode.
- · `2C-two-columns` Split current window vertically for two-column editing.

## 5x5

- · `5x5` Play 5x5.
- · `5x5-crack` Attempt to find a solution for 5x5.
- · `5x5-crack-mutating-best` Attempt to crack 5x5 by mutating the best solution.
- · `5x5-crack-mutating-current` Attempt to crack 5x5 by mutating the current solution.
- · `5x5-crack-randomly` Attempt to crack 5x5 using random solutions.
- · `5x5-crack-xor-mutate` Attempt to crack 5x5 by xoring the current and best solution.

## abbrev

- · `abbrev-edit-save-buffer` Save all the user-level abbrev definitions in current buffer.
- · `abbrev-edit-save-to-file` Save to FILE all the user-level abbrev definitions in current buffer.
- · `abbrev-mode` Toggle Abbrev mode in the current buffer.
- · `abbrev-prefix-mark` Mark point as the beginning of an abbreviation.
- · `abbrev-suggest-show-report` Show a buffer with the list of abbrevs you could have used.

## abort

- · `abort-minibuffers` Abort the current minibuffer.
- ✓ `abort-recursive-edit` Abort the command that requested this recursive edit or minibuffer input.

## about

- ✓ `about-emacs` Display the *About GNU Emacs* buffer.

## accelerate

- · `accelerate-menu` Start key navigation of the menu bar in FRAME.

## activate

- · `activate-transient-input-method` Select and enable a transient input method for the current buffer.

## ad

- · `ad-activate` Activate all the advice information of an advised FUNCTION.
- · `ad-disable-advice` Disable the advice of FUNCTION with CLASS and NAME.
- · `ad-enable-advice` Enables the advice of FUNCTION with CLASS and NAME.

## add

- · `add-change-log-entry` Find ChangeLog buffer, add an entry for today and an item for this file.
- · `add-change-log-entry-other-window` Find change log file in other window and add entry and item.
- · `add-dir-local-variable` Add directory-local VARIABLE with its VALUE and MODE to .dir-locals.el.
- · `add-file-local-variable` Add file-local VARIABLE with its VALUE to the Local Variables list.
- · `add-file-local-variable-prop-line` Add file-local VARIABLE with its VALUE to the -*- line.
- · `add-global-abbrev` Define a global (all modes) abbrev whose expansion is last word before point.
- · `add-mode-abbrev` Define a mode-specific abbrev whose expansion is the last word before point.
- · `add-name-to-file` Give FILE additional name NEWNAME.  Both args must be strings.

## advice

- · `advice-remove` Like `remove-function' but for the function named SYMBOL.

## align

- · `align` Attempt to align a region based on a set of alignment rules.
- · `align-current` Call `align' on the current alignment section.
- · `align-entire` Align the selected region as if it were one alignment section.
- · `align-highlight-rule` Highlight the whitespace which a given rule would have modified.
- · `align-newline-and-indent` A replacement function for `newline-and-indent', aligning as it goes.
- · `align-regexp` Align the current region using an ad-hoc rule read from the minibuffer.
- · `align-unhighlight-rule` Remove any highlighting that was added by `align-highlight-rule'.

## allout

- · `allout-mode` Toggle Allout outline mode.
- · `allout-outlinify-sticky` Activate outline mode and establish file var so it is started subsequently.
- · `allout-widgets-mode` Toggle Allout Widgets mode.

## analyze

- · `analyze-text-conversion` Analyze the results of the previous text conversion event.

## ange

- · `ange-ftp-re-read-dir` Reread remote directory DIR to update the directory cache.
- · `ange-ftp-reread-dir` Reread remote directory DIR to update the directory cache.

## animate

- · `animate-birthday-present` Return a birthday present in the buffer *Birthday-Present*.

## ansi

- · `ansi-color-for-comint-mode-on` Set `ansi-color-for-comint-mode' to t.
- · `ansi-term` Start a terminal-emulator in a new buffer.

## antlr

- · `antlr-mode` Major mode for editing ANTLR grammar files.
- · `antlr-show-makefile-rules` Show Makefile rules for all grammar files in the current directory.
- · `antlr-v4-mode` Major mode for editing ANTLR v4 grammar files.

## append

- ✓ `append-next-kill` Cause following command, if it kills, to add to previous kill.
- · `append-to-buffer` Append to specified BUFFER the text of the region.
- · `append-to-file` Append the contents of the region to the end of file FILENAME.
- ✓ `append-to-register` Append region of text between START and END to REGISTER.

## apply

- · `apply-macro-to-region-lines` Apply last keyboard macro to all lines in the region.

## appt

- · `appt-activate` Toggle checking of appointments.
- · `appt-add` Add an appointment for today at TIME with message MSG.

## apropos

- · `apropos` Show all meaningful Lisp symbols whose names match PATTERN.
- ✓ `apropos-command` Show commands (interactively callable functions) that match PATTERN.
- ✓ `apropos-documentation` Show symbols whose documentation contains matches for PATTERN.
- · `apropos-function` Show functions that match PATTERN.
- · `apropos-library` List the variables and functions defined by library FILE.
- · `apropos-local-value` Show buffer-local variables whose values match PATTERN.
- · `apropos-local-variable` Show buffer-local variables that match PATTERN.
- ✓ `apropos-user-option` Show user options that match PATTERN.
- · `apropos-value` Show all symbols whose value's printed representation matches PATTERN.
- · `apropos-variable` Show variables that match PATTERN.

## arp

- · `arp` Run `arp-program' and display diagnostic output.

## array

- · `array-mode` Major mode for editing arrays.

## artist

- · `artist-mode` Toggle Artist mode.

## asm

- · `asm-mode` Major mode for editing typical assembler code.

## async

- ✓ `async-shell-command` Execute string COMMAND asynchronously in background.

## authinfo

- · `authinfo-mode` Mode for editing .authinfo/.netrc files.

## auto

- · `auto-composition-mode` Toggle Auto Composition mode.
- · `auto-compression-mode` Toggle Auto Compression mode.
- · `auto-encryption-mode` Toggle automatic file encryption/decryption (Auto Encryption mode).
- · `auto-fill-mode` Toggle automatic line breaking (Auto Fill mode).
- · `auto-image-file-mode` Toggle visiting of image files as images (Auto Image File mode).
- · `auto-insert` Insert default contents into new files if variable `auto-insert' is non-nil.
- · `auto-insert-mode` Toggle Auto-insert mode, a global minor mode.
- · `auto-lower-mode` Toggle whether or not the selected frame should auto-lower.
- · `auto-raise-mode` Toggle whether or not selected frames should auto-raise.
- · `auto-revert-mode` Toggle reverting buffer when the file changes (Auto-Revert Mode).
- · `auto-revert-tail-mode` Toggle reverting tail of buffer when the file grows.
- · `auto-save-mode` Toggle auto-saving in the current buffer (Auto Save mode).
- · `auto-save-visited-mode` Toggle automatic saving of file-visiting buffers to their files.

## autoconf

- · `autoconf-mode` Major mode for editing Autoconf configure.ac files.

## awk

- · `awk-mode` Major mode for editing AWK code.

## back

- ✓ `back-to-indentation` Move point to the first non-whitespace character on this line.

## backward

- · `backward-button` Move to the Nth previous button, or Nth next button if N is negative.
- ✓ `backward-char` Move point N characters backward (forward if N is negative).
- · `backward-delete-char` Delete the previous N characters (following if N is negative).
- ✓ `backward-delete-char-untabify` Delete characters backward, changing tabs into spaces.
- · `backward-kill-paragraph` Kill back to start of paragraph.
- ✓ `backward-kill-sentence` Kill back from point to start of sentence.
- ✓ `backward-kill-sexp` Kill the sexp (balanced expression) preceding point.
- ✓ `backward-kill-word` Kill characters backward until encountering the beginning of a word.
- ✓ `backward-list` Move backward across one balanced group of parentheses.
- ✓ `backward-page` Move backward to page boundary.  With arg, repeat, or go fwd if negative.
- ✓ `backward-paragraph` Move backward to start of paragraph.
- ✓ `backward-sentence` Move backward to start of sentence.
- ✓ `backward-sexp` Move backward across one balanced expression (sexp).
- · `backward-to-indentation` Move backward ARG lines and position at first nonblank character.
- · `backward-to-word` Move backward until encountering the end of the ARGth word.
- ✓ `backward-up-list` Move backward out of one level of parentheses.
- ✓ `backward-word` Move backward until encountering the beginning of a word.

## balance

- ✓ `balance-windows` Balance the sizes of windows shown on the selected frame.
- · `balance-windows-area` Make all visible windows the same area (approximately).

## base64

- · `base64-decode-region` Base64-decode the region between BEG and END.
- · `base64-encode-region` Base64-encode the region between BEG and END.

## base64url

- · `base64url-encode-region` Base64url-encode the region between BEG and END.

## bash

- · `bash-ts-mode` Major mode for editing Bash shell scripts.

## basic

- · `basic-save-buffer` Save the current buffer in its visited file, if it has been modified.

## bat

- · `bat-mode` Major mode for editing DOS/Windows batch files.

## battery

- · `battery` Display battery status information in the echo area.

## beginning

- ✓ `beginning-of-buffer` Move point to the beginning of the buffer.
- ✓ `beginning-of-buffer-other-window` Move point to the beginning of the buffer in the other window.
- ✓ `beginning-of-defun` Move backward to the beginning of a defun.
- · `beginning-of-defun-comments` Move to the beginning of ARGth defun, including comments.
- · `beginning-of-defun-raw` Move point to the character that starts a defun.
- ✓ `beginning-of-line` Move point to beginning of current line (in the logical order).
- · `beginning-of-line-text` Move to the beginning of the text on this line.
- · `beginning-of-visual-line` Move point to beginning of current visual line.

## benchmark

- · `benchmark` Print the time taken for REPETITIONS executions of FORM.

## bibtex

- · `bibtex-initialize` (Re)Initialize BibTeX buffers.
- · `bibtex-mode` Major mode for editing BibTeX files.
- · `bibtex-search-entry` Move point to the beginning of BibTeX entry named KEY.
- · `bibtex-style-mode` Major mode for editing BibTeX style files.

## binary

- · `binary-overwrite-mode` Toggle Binary Overwrite mode.

## binhex

- · `binhex-decode-region` Binhex decode region between START and END.
- · `binhex-decode-region-external` Binhex decode region between START and END using external decoder.
- · `binhex-decode-region-internal` Binhex decode region between START and END without using an external program.

## blackbox

- · `blackbox` Play blackbox.

## blink

- · `blink-cursor-mode` Toggle cursor blinking (Blink Cursor mode).
- · `blink-matching-open` Momentarily highlight the beginning of the sexp before point.

## bookmark

- ✓ `bookmark-bmenu-list` Display a list of existing bookmarks.
- · `bookmark-bmenu-search` Incremental search of bookmarks, hiding the non-matches as we go.
- · `bookmark-delete` Delete BOOKMARK-NAME from the bookmark list.
- · `bookmark-delete-all` Permanently delete all bookmarks.
- · `bookmark-insert` Insert the text of the file pointed to by bookmark BOOKMARK-NAME.
- · `bookmark-insert-location` Insert the name of the file associated with BOOKMARK-NAME.
- ✓ `bookmark-jump` Jump to bookmark BOOKMARK (a point in some file).
- · `bookmark-jump-other-frame` Jump to BOOKMARK in another frame.  See `bookmark-jump' for more.
- · `bookmark-jump-other-window` Jump to BOOKMARK in another window.  See `bookmark-jump' for more.
- · `bookmark-load` Load bookmarks from FILE (which must be in bookmark format).
- · `bookmark-locate` Insert the name of the file associated with BOOKMARK-NAME.
- · `bookmark-relocate` Relocate BOOKMARK-NAME to another file, reading file name with minibuffer.
- · `bookmark-rename` Change the name of OLD-NAME bookmark to NEW-NAME name.
- · `bookmark-save` Save currently defined bookmarks in FILE.
- ✓ `bookmark-set` Set a bookmark named NAME at the current location.
- ✓ `bookmark-set-no-overwrite` Set a bookmark named NAME at the current location.
- · `bookmark-write` Write bookmarks to a file (reading the file name with the minibuffer).

## bovine

- · `bovine-grammar-mode` Major mode for editing Bovine grammars.

## browse

- · `browse-url` Open URL using a configurable method.
- · `browse-url-at-mouse` Use a web browser to load a URL clicked with the mouse.
- · `browse-url-at-point` Open URL at point using a configurable method.
- · `browse-url-button-open` Follow the link under point using `browse-url'.
- · `browse-url-chromium` Ask the Chromium WWW browser to load URL.
- · `browse-url-default-android-browser` Browse URL with the system default browser.
- · `browse-url-default-gtk-browser` Browse URL with GTK's idea of the default browser.
- · `browse-url-default-haiku-browser` Browse URL with the system default browser.
- · `browse-url-elinks` Ask the Elinks WWW browser to load URL.
- · `browse-url-emacs` Ask Emacs to load URL into a buffer and show it in another window.
- · `browse-url-firefox` Ask the Firefox WWW browser to load URL.
- · `browse-url-generic` Ask the WWW browser defined by `browse-url-generic-program' to load URL.
- · `browse-url-kde` Ask the KDE WWW browser to load URL.
- · `browse-url-mail` Open a new mail message buffer within Emacs for the RFC 2368 URL.
- · `browse-url-mozilla` Ask the Mozilla WWW browser to load URL.
- · `browse-url-of-buffer` Use a web browser to display BUFFER.
- · `browse-url-of-dired-file` In Dired, ask a WWW browser to display the file named on this line.
- · `browse-url-of-file` Use a web browser to display FILE.
- · `browse-url-of-region` Use a web browser to display the current region.
- · `browse-url-text-emacs` Ask a text browser to load URL.
- · `browse-url-text-xterm` Ask a text browser to load URL.
- · `browse-url-w3` Ask the w3 WWW browser to load URL.
- · `browse-url-webpositive` Ask the WebPositive WWW browser to load URL.
- · `browse-url-with-browser-kind` Browse URL with a browser of the given browser KIND.
- · `browse-url-xdg-open` Pass the specified URL to the "xdg-open" command.
- · `browse-web` Fetch URL and render the page.

## bs

- · `bs-customize` Customization of group bs for Buffer Selection Menu.
- · `bs-cycle-next` Select next buffer defined by buffer cycling.
- · `bs-cycle-previous` Select previous buffer defined by buffer cycling.
- · `bs-show` Make a menu of buffers so you can manipulate buffers or the buffer list.

## bubbles

- · `bubbles` Play Bubbles game.

## buffer

- · `buffer-disable-undo` Make BUFFER stop keeping undo information.
- · `buffer-enable-undo` Start keeping undo information for buffer BUFFER.
- · `buffer-face-mode` Minor mode for a buffer-specific default face.
- · `buffer-face-set` Enable `buffer-face-mode', using face specs SPECS.
- · `buffer-face-toggle` Toggle `buffer-face-mode', using face specs SPECS.
- · `buffer-menu` Switch to the Buffer Menu.
- · `buffer-menu-open` Start key navigation of the buffer menu.
- · `buffer-menu-other-window` Display the Buffer Menu in another window.
- · `buffer-to-register` Store reference to BUFFER in REGISTER.

## Buffer

- · `Buffer-menu-1-window` Select this line's buffer, alone, in full frame.
- · `Buffer-menu-2-window` Select this line's buffer, with previous buffer in second window.
- · `Buffer-menu-backup-unmark` Move up and cancel all requested operations on buffer on line above.
- · `Buffer-menu-bury` Bury the buffer listed on this line.
- · `Buffer-menu-delete` Mark the buffer on this Buffer Menu buffer line for deletion.
- · `Buffer-menu-delete-backwards` Mark the buffer on this Buffer Menu line for deletion, and move up.
- · `Buffer-menu-execute` Save and/or delete marked buffers in the Buffer Menu.
- · `Buffer-menu-isearch-buffers` Search for a string through all marked buffers using Isearch.
- · `Buffer-menu-isearch-buffers-regexp` Search for a regexp through all marked buffers using Isearch.
- · `Buffer-menu-mark` Mark the Buffer menu entry at point for later display.
- · `Buffer-menu-mode-menu` Menu for `Buffer-menu-mode' buffers.
- · `Buffer-menu-mouse-select` Select the buffer whose line you click on.
- · `Buffer-menu-multi-occur` Show all lines in marked buffers containing a match for a regexp.
- · `Buffer-menu-not-modified` Mark the buffer on this line as unmodified (no changes to save).
- · `Buffer-menu-other-window` Select this line's buffer in other window, leaving buffer menu visible.
- · `Buffer-menu-save` Mark the buffer on this Buffer Menu line for saving.
- · `Buffer-menu-select` Select this line's buffer; also, display buffers marked with `>'.
- · `Buffer-menu-sort` Sort Tabulated List entries by the column at point.
- · `Buffer-menu-switch-other-window` Make the other window select this line's buffer.
- · `Buffer-menu-this-window` Select this line's buffer in this window.
- · `Buffer-menu-toggle-files-only` Toggle whether the current `buffer-menu' displays only file buffers.
- · `Buffer-menu-toggle-internal` Toggle whether the current `buffer-menu' displays internal buffers.
- · `Buffer-menu-toggle-read-only` Toggle read-only status of buffer on this line.
- · `Buffer-menu-unmark` Cancel all requested operations on buffer on this line and move down.
- · `Buffer-menu-unmark-all` Cancel all requested operations on buffers.
- · `Buffer-menu-unmark-all-buffers` Cancel a requested operation on all buffers.
- · `Buffer-menu-view` View this line's buffer in View mode.
- · `Buffer-menu-view-other-window` View this line's buffer in View mode in another window.
- · `Buffer-menu-visit-tags-table` Visit the tags table in the buffer on this line.  See `visit-tags-table'.

## bug

- · `bug-reference-mode` Toggle hyperlinking bug references in the buffer (Bug Reference mode).
- · `bug-reference-prog-mode` Like `bug-reference-mode', but only buttonize in comments and strings.

## bury

- · `bury-buffer` Put BUFFER-OR-NAME at the end of the list of all buffers.

## butterfly

- · `butterfly` Use butterflies to flip the desired bit on the drive platter.

## button

- · `button-describe` Display a buffer with information about the button at point.
- · `button-mode` A minor mode for navigating to buttons with the TAB key.

## byte

- · `byte-compile-file` Compile a file of Lisp code named FILENAME into a file of byte code.
- · `byte-force-recompile` Recompile every `.el' file in DIRECTORY that already has a `.elc' file.
- · `byte-recompile-directory` Recompile every `.el' file in DIRECTORY that needs recompilation.

## c

- · `c-add-style` Add a style to `c-style-alist', or update an existing one.
- · `c-guess` Guess the style in the region up to `c-guess-region-max', and install it.
- · `c-guess-buffer` Guess the style on the whole current buffer, and install it.
- · `c-guess-buffer-no-install` Guess the style on the whole current buffer; don't install it.
- · `c-guess-install` Install the latest guessed style into the current buffer.
- · `c-guess-no-install` Guess the style in the region up to `c-guess-region-max'; don't install it.
- · `c-guess-region` Guess the style on the region and install it.
- · `c-guess-region-no-install` Guess the style on the region; don't install it.
- · `c-macro-expand` Expand C macros in the region, using the C preprocessor.
- · `c-mode` Major mode for editing C code.
- · `c-or-c++-mode` Analyze buffer and enable either C or C++ mode.
- · `c-or-c++-ts-mode` Analyze buffer and enable either C or C++ mode.
- · `c-set-offset` Change the value of a syntactic element symbol in `c-offsets-alist'.
- · `c-set-style` Set the current buffer to use the style STYLENAME.
- · `c-ts-mode` Major mode for editing C, powered by tree-sitter.

## c++

- · `c++-mode` Major mode for editing C++ code.
- · `c++-ts-mode` Major mode for editing C++, powered by tree-sitter.

## calc

- · `calc` The Emacs Calculator.  Full documentation is listed under `calc-mode'.
- · `calc-dispatch` Invoke the GNU Emacs Calculator.  See \[calc-dispatch-help] for details.
- · `calc-embedded` Start Calc Embedded mode on the formula surrounding point.
- · `calc-embedded-activate` Scan the current editing buffer for all embedded := and => formulas.
- · `calc-grab-rectangle` Parse a rectangle as a matrix of numbers and push it on the Calculator stack.
- · `calc-grab-region` Parse the region as a vector of numbers and push it on the Calculator stack.
- · `calc-grab-sum-across` Parse a rectangle as a matrix of numbers and sum its rows.
- · `calc-grab-sum-down` Parse a rectangle as a matrix of numbers and sum its columns.
- · `calc-keypad` Invoke the Calculator in "visual keypad" mode.
- · `calc-undo` 

## calculator

- · `calculator` Run the Emacs calculator.

## calendar

- · `calendar` Display a three-month Gregorian calendar.
- · `calendar-hebrew-list-yahrzeits` List Yahrzeit dates for *Gregorian* DEATH-DATE from START-YEAR to END-YEAR.

## call

- ✓ `call-last-kbd-macro` Call the last keyboard macro that you defined with \[start-kbd-macro].

## cancel

- · `cancel-debug-on-entry` Undo effect of \[debug-on-entry] on FUNCTION.
- · `cancel-debug-on-variable-change` Undo effect of \[debug-on-variable-change] on VARIABLE.
- · `cancel-debug-watch` Undo effect of \[debug-on-variable-change] on VARIABLE.
- · `cancel-function-timers` Cancel all timers which would run FUNCTION.

## canlock

- · `canlock-verify` Verify Cancel-Lock or Cancel-Key in BUFFER.

## canonically

- · `canonically-space-region` Remove extra spaces between words in region.

## capitalize

- · `capitalize-dwim` Capitalize words in the region, if active; if not, capitalize word at point.
- · `capitalize-region` Convert the region to capitalized form.
- ✓ `capitalize-word` Capitalize from point to the end of word, moving over.

## capitalized

- · `capitalized-words-mode` Toggle subword movement and editing (Subword mode).

## cd

- · `cd` Make DIR become the current buffer's default directory.

## center

- · `center-line` Center the line point is on, within the width specified by `fill-column'.
- · `center-line-mode` Minor mode for keeping modified lines centered horizontally.
- · `center-paragraph` Center each nonblank line in the paragraph at or after point.
- · `center-region` Center each nonblank line starting in the region.

## cfengine

- · `cfengine-auto-mode` Choose `cfengine2-mode' or `cfengine3-mode' by buffer contents.

## cfengine2

- · `cfengine2-mode` Major mode for editing CFEngine2 input.

## cfengine3

- · `cfengine3-mode` Major mode for editing CFEngine3 input.

## change

- · `change-log-merge` Merge the contents of change log file OTHER-LOG with this buffer.
- · `change-log-mode` Major mode for editing change logs; like Indented Text mode.

## cheat

- · `cheat-sheet` Display a quick-help buffer showing popular commands and their bindings.

## check

- · `check-declare-directory` Check veracity of all `declare-function' statements under directory ROOT.
- · `check-declare-file` Check veracity of all `declare-function' statements in FILE.
- · `check-parens` Check for unbalanced parentheses in the current buffer.

## checkdoc

- · `checkdoc` Interactively check the entire buffer for style errors.
- · `checkdoc-comments` Find missing comment sections in the current Emacs Lisp file.
- · `checkdoc-continue` Find the next doc string in the current buffer which has a style error.
- · `checkdoc-current-buffer` Check current buffer for document, comment, error style, and rogue spaces.
- · `checkdoc-defun` Examine the doc string of the function or variable under point.
- · `checkdoc-dired` In Dired, run `checkdoc' on marked files.
- · `checkdoc-eval-current-buffer` Evaluate and check documentation for the current buffer.
- · `checkdoc-eval-defun` Evaluate the current form with `eval-defun' and check its documentation.
- · `checkdoc-interactive` Interactively check the current buffer for doc string errors.
- · `checkdoc-ispell` Check the style and spelling of everything interactively.
- · `checkdoc-ispell-comments` Check the style and spelling of the current buffer's comments.
- · `checkdoc-ispell-continue` Check the style and spelling of the current buffer after point.
- · `checkdoc-ispell-current-buffer` Check the style and spelling of the current buffer.
- · `checkdoc-ispell-defun` Check the style and spelling of the current defun with Ispell.
- · `checkdoc-ispell-interactive` Check the style and spelling of the current buffer interactively.
- · `checkdoc-ispell-message-interactive` Check the style and spelling of message text interactively.
- · `checkdoc-ispell-message-text` Check the style and spelling of message text interactively.
- · `checkdoc-ispell-start` Check the style and spelling of the current buffer.
- · `checkdoc-message-interactive` Interactively check the current buffer for message string errors.
- · `checkdoc-message-text` Scan the buffer for occurrences of the error function, and verify text.
- · `checkdoc-minor-mode` Toggle automatic docstring checking (Checkdoc minor mode).
- · `checkdoc-package-keywords` Find package keywords that aren't in `finder-known-keywords'.
- · `checkdoc-rogue-spaces` Find extra spaces at the end of lines in the current file.
- · `checkdoc-start` Start scanning the current buffer for documentation string style errors.

## chmod

- · `chmod` Set mode bits of file named FILENAME to MODE (an integer).

## choose

- · `choose-completion` Choose the completion at point.

## cl

- · `cl-font-lock-built-in-mode` Highlight built-in functions, variables, and types in `lisp-mode'.
- · `cl-old-struct-compat-mode` Enable backward compatibility with old-style structs.

## clean

- · `clean-buffer-list` Kill old buffers that have not been displayed recently.
- · `clean-mode` A mode that removes all overlays and text properties.

## clear

- ✓ `clear-rectangle` Blank out the region-rectangle.

## clipboard

- · `clipboard-kill-region` Kill the region, and save it in the GUI's clipboard.
- · `clipboard-kill-ring-save` Copy region to kill ring, and save in the GUI's clipboard.
- · `clipboard-yank` Insert the clipboard contents, or the last stretch of killed text.

## clone

- · `clone-buffer` Create and return a twin copy of the current buffer.
- · `clone-frame` Make a new frame with the same parameters and windows as FRAME.
- · `clone-indirect-buffer` Create an indirect buffer that is a twin copy of the current buffer.
- · `clone-indirect-buffer-other-window` Like `clone-indirect-buffer' but display in another window.

## close

- · `close-display-connection` Close the connection to a display, deleting all its associated frames.
- · `close-rectangle` Delete all whitespace following a specified column in each line.

## cmake

- · `cmake-ts-mode` Major mode for editing CMake files, powered by tree-sitter.

## column

- · `column-number-mode` Toggle column number display in the mode line (Column Number mode).

## comint

- · `comint-redirect-send-command` Send COMMAND to process in current buffer, with output to OUTPUT-BUFFER.
- · `comint-redirect-send-command-to-process` Send COMMAND to PROCESS, with output to OUTPUT-BUFFER.
- · `comint-run` Run PROGRAM in a Comint buffer and switch to that buffer.

## command

- · `command-apropos` Show commands (interactively callable functions) that match PATTERN.
- · `command-history` Examine commands from variable `command-history' in a buffer.

## comment

- · `comment-box` Comment out the BEG .. END region, putting it inside a box.
- ✓ `comment-dwim` Call the comment command you want (Do What I Mean).
- · `comment-indent` Indent this line's comment to `comment-column', or insert an empty comment.
- · `comment-indent-new-line` Break line at point and indent, continuing comment if within one.
- · `comment-kill` Kill the first comment on this line, if any.
- ✓ `comment-line` Comment or uncomment current line and leave point after it.
- · `comment-or-uncomment-region` Call `comment-region', unless the region only consists of comments,
- · `comment-region` Comment or uncomment each line in the region.
- · `comment-set-column` Set the comment column based on point.

## common

- · `common-lisp-mode` Major mode for editing programs in Common Lisp and other similar Lisps.

## compare

- · `compare-windows` Compare text in current window with text in another window.

## compilation

- · `compilation-minor-mode` Toggle Compilation minor mode.
- · `compilation-mode` Major mode for compilation log buffers.
- · `compilation-next-error-function` Advance to the next error message and visit the file where the error was.
- · `compilation-shell-minor-mode` Toggle Compilation Shell minor mode.

## compile

- ✓ `compile` Compile the program including the current buffer.  Default: run `make'.
- · `compile-defun` Compile and evaluate the current top-level form.

## complete

- · `complete-symbol` Perform completion on the text around point.
- · `complete-tag` Perform tags completion on the text around point.

## completion

- · `completion-at-point` Perform completion on the text around point.
- · `completion-help-at-point` Display the completions on the text around point.
- · `completion-in-region-mode` Transient minor mode used during `completion-in-region'.
- · `completion-list-mode` Major mode for buffers showing lists of possible completions.
- · `completion-preview-mode` Show in-buffer completion suggestions in a preview as you type.

## compose

- · `compose-last-chars` Compose last characters.
- · `compose-mail` Start composing a mail message to send.
- · `compose-mail-other-frame` Like \[compose-mail], but edit the outgoing message in another frame.
- · `compose-mail-other-window` Like \[compose-mail], but edit the outgoing message in another window.
- · `compose-region` Compose characters in the current region.

## conf

- · `conf-colon-mode` Conf Mode starter for Colon files.
- · `conf-desktop-mode` Conf Mode started for freedesktop.org Desktop files.
- · `conf-javaprop-mode` Conf Mode starter for Java properties files.
- · `conf-mode` Mode for Unix and Windows Conf files and Java properties.
- · `conf-npmrc-mode` Major mode derived from `conf-mode' by `define-derived-mode'.
- · `conf-ppd-mode` Conf Mode starter for Adobe/CUPS PPD files.
- · `conf-space-keywords` Enter Conf Space mode using regexp KEYWORDS to match the keywords.
- · `conf-space-mode` Conf Mode starter for space separated conf files.
- · `conf-toml-mode` Conf Mode starter for TOML files.
- · `conf-unix-mode` Conf Mode starter for Unix style Conf files.
- · `conf-windows-mode` Conf Mode starter for Windows style Conf files.
- · `conf-xdefaults-mode` Conf Mode starter for Xdefaults files.

## context

- · `context-menu-mode` Toggle Context Menu mode.
- · `context-menu-open` Start key navigation of the context menu.

## cookie

- · `cookie` Return a random phrase from PHRASE-FILE.

## copy

- · `copy-directory` Copy DIRECTORY to NEWNAME.  Both args must be strings.
- · `copy-dir-locals-to-file-locals` Copy directory-local variables to the Local Variables list.
- · `copy-dir-locals-to-file-locals-prop-line` Copy directory-local variables to the -*- line.
- · `copy-file` Copy FILE to NEWNAME.  Both args must be strings.
- · `copy-file-locals-to-dir-locals` Copy file-local variables to .dir-locals.el.
- · `copy-from-above-command` Copy characters from previous nonblank line, starting just above point.
- · `copy-matching-lines` Copy lines containing matches for REGEXP to the kill ring.
- ✓ `copy-rectangle-as-kill` Copy the region-rectangle and save it as the last killed one.
- ✓ `copy-rectangle-to-register` Copy rectangular region of text between START and END into REGISTER.
- ✓ `copy-region-as-kill` Save the region as if killed, but don't kill it.
- · `copy-theme-options` Copy settings from THEME into the user theme.
- · `copy-to-buffer` Copy to specified BUFFER the text of the region.
- ✓ `copy-to-register` Copy region of text between START and END into REGISTER.

## copyright

- · `copyright` Insert a copyright by $ORGANIZATION notice at cursor.
- · `copyright-fix-years` Convert 2 digit years to 4 digit years.
- · `copyright-update` Update copyright notice to indicate the current year.
- · `copyright-update-directory` Update copyright notice for all files in DIRECTORY matching MATCH.

## count

- ✓ `count-lines-page` Report number of lines on current page, and how many are before or after point.
- · `count-matches` Print and return number of matches for REGEXP following point.
- ✓ `count-words` Count words between START and END.
- ✓ `count-words-region` Count the number of words in the region.

## cperl

- · `cperl-mode` Major mode for editing Perl code.
- · `cperl-perldoc` Run `perldoc' on WORD.
- · `cperl-perldoc-at-point` Run a `perldoc' on the word around point.

## cpp

- · `cpp-highlight-buffer` Highlight C code according to preprocessor conditionals.
- · `cpp-parse-edit` Edit display information for cpp conditionals.

## csharp

- · `csharp-mode` Major mode for editing Csharp code.
- · `csharp-ts-mode` Major mode for editing C# code.

## css

- · `css-lookup-symbol` Display the CSS documentation for SYMBOL, as found on MDN.
- · `css-mode` Major mode to edit Cascading Style Sheets (CSS).
- · `css-ts-mode` Major mode to edit Cascading Style Sheets (CSS).

## cua

- · `cua-mode` Toggle Common User Access style editing (CUA mode).
- · `cua-rectangle-mark-mode` Toggle the region as rectangular.
- · `cua-selection-mode` Enable CUA selection mode without the C-z/C-x/C-c/C-v bindings.

## cursor

- · `cursor-face-highlight-mode` When enabled, highlight text that has `cursor-face' property near point.
- · `cursor-intangible-mode` Keep cursor outside of any `cursor-intangible' text property.
- · `cursor-sensor-mode` Handle the `cursor-sensor-functions' text property.

## custom

- · `custom-theme-visit-theme` Set up a Custom buffer to edit custom theme THEME.

## customize

- · `customize` Select a customization buffer which you can use to set user options.
- · `customize-apropos` Customize loaded options, faces and groups matching PATTERN.
- · `customize-apropos-faces` Customize all loaded faces matching REGEXP.
- · `customize-apropos-groups` Customize all loaded groups matching REGEXP.
- · `customize-apropos-options` Customize all loaded customizable options matching REGEXP.
- · `customize-browse` Create a tree browser for the customize hierarchy.
- · `customize-changed` Customize all settings whose meanings have changed in Emacs itself.
- · `customize-changed-options` Customize all settings whose meanings have changed in Emacs itself.
- · `customize-create-theme` Create or edit a custom theme.
- · `customize-dirlocals` Customize Directory Local Variables in the current directory.
- · `customize-face` Customize FACE, which should be a face name or nil.
- · `customize-face-other-window` Show customization buffer for face FACE in other window.
- · `customize-group` Customize GROUP, which must be a customization group.
- · `customize-group-other-window` Customize GROUP, which must be a customization group, in another window.
- · `customize-icon` Customize ICON.
- · `customize-mode` Customize options related to a major or minor mode.
- · `customize-option` Customize SYMBOL, which must be a user option.
- · `customize-option-other-window` Customize SYMBOL, which must be a user option.
- · `customize-rogue` Customize all user variables modified outside customize.
- · `customize-save-customized` Save all user options which have been set in this session.
- · `customize-saved` Customize all saved options and faces.
- · `customize-save-variable` Set the default for VARIABLE to VALUE, and save it for future sessions.
- · `customize-set-value` Set VARIABLE to VALUE, and return VALUE.  VALUE is a Lisp object.
- · `customize-set-variable` Set the default for VARIABLE to VALUE, and return VALUE.
- · `customize-themes` Display a selectable list of Custom themes.
- · `customize-toggle-option` Toggle the value of boolean option SYMBOL for this session.
- · `customize-unsaved` Customize all options and faces set in this session but not saved.
- · `customize-variable` Customize SYMBOL, which must be a user option.
- · `customize-variable-other-window` Customize SYMBOL, which must be a user option.

## cvs

- · `cvs-checkout` Run a `cvs checkout MODULES' in DIR.
- · `cvs-examine` Run a `cvs -n update' in the specified DIRECTORY.
- · `cvs-quickdir` Open a *cvs* buffer on DIR without running cvs.
- · `cvs-status` Run a `cvs status' in the current working DIRECTORY.
- · `cvs-status-mode` Mode used for cvs status output.
- · `cvs-update` Run a `cvs update' in the current working DIRECTORY.

## cwarn

- · `cwarn-mode` Minor mode that highlights suspicious C and C++ constructions.

## cycle

- ✓ `cycle-spacing` Manipulate whitespace around point in a smart way.

## dabbrev

- ✓ `dabbrev-completion` Completion on current word.
- ✓ `dabbrev-expand` Expand previous word "dynamically".

## dbus

- · `dbus-handle-event` Handle events from the D-Bus.
- · `dbus-monitor` Invoke `dbus-register-monitor' interactively, and switch to the buffer.

## dbx

- · `dbx` Run dbx on program FILE in buffer *gud-FILE*.

## dcl

- · `dcl-mode` Major mode for editing DCL-files.

## debug

- · `debug` Enter debugger.  \<debugger-mode-map>`\[debugger-continue]' returns from the debugger.
- · `debug-on-entry` Request FUNCTION to invoke debugger each time it is called.
- · `debug-on-variable-change` Trigger a debugger invocation when VARIABLE is changed.
- · `debug-watch` Trigger a debugger invocation when VARIABLE is changed.

## debugger

- · `debugger-trap` Stop Emacs and hand over control to GDB.

## decipher

- · `decipher` Format a buffer of ciphertext for cryptanalysis and enter Decipher mode.
- · `decipher-mode` Major mode for decrypting monoalphabetic substitution ciphers.

## decode

- · `decode-coding-region` Decode the current region using the specified coding system.
- · `decode-hz-buffer` Decode HZ/ZW encoded text in the current buffer.
- · `decode-hz-region` Decode HZ/ZW encoded text in the current region.

## decompose

- · `decompose-region` Decompose text in the current region.

## decrease

- · `decrease-left-margin` Make the left margin of the region smaller.
- · `decrease-right-margin` Make the right margin of the region smaller.

## default

- ✓ `default-indent-new-line` Break line at point and indent.

## define

- · `define-abbrevs` Define abbrevs according to current visible buffer contents.
- · `define-global-abbrev` Define ABBREV as a global abbreviation that expands into EXPANSION.
- · `define-mail-abbrev` Define NAME as a mail alias abbrev that translates to DEFINITION.
- · `define-mail-alias` Define NAME as a mail alias that translates to DEFINITION.
- · `define-mode-abbrev` Define ABBREV as a mode-specific abbreviation that expands into EXPANSION.

## defining

- · `defining-kbd-macro` Record subsequent keyboard input, defining a keyboard macro.

## delete

- · `delete-active-region` Delete the active region.
- · `delete-all-space` Delete all spaces, tabs, and newlines around point.
- ✓ `delete-backward-char` Delete the previous N characters (following if N is negative).
- ✓ `delete-blank-lines` On blank line, delete all surrounding blank lines, leaving just one.
- ✓ `delete-char` Delete the following N characters (previous if N is negative).
- · `delete-completion-window` Delete the completion list window.
- · `delete-directory` Delete the directory named DIRECTORY.  Does not follow symlinks.
- · `delete-dir-local-variable` Delete all MODE settings of dir-local VARIABLE from .dir-locals.el.
- ✓ `delete-duplicate-lines` Delete all but one copy of any identical lines in the region.
- · `delete-file` Delete file named FILENAME.  If it is a symlink, remove the symlink.
- · `delete-file-local-variable` Delete all settings of file-local VARIABLE from the Local Variables list.
- · `delete-file-local-variable-prop-line` Delete all settings of file-local VARIABLE from the -*- line.
- ✓ `delete-forward-char` Delete the following N characters (previous if N is negative).
- · `delete-frame` Delete FRAME, eliminating it from use.
- ✓ `delete-horizontal-space` Delete all spaces and tabs around point.
- ✓ `delete-indentation` Join this line to previous and fix up whitespace at join.
- · `delete-matching-lines` Delete lines containing matches for REGEXP.
- · `delete-minibuffer-contents` Delete all user input in a minibuffer.
- · `delete-non-matching-lines` Delete all lines except those containing matches for REGEXP.
- · `delete-other-frames` Delete all frames on FRAME's terminal, except FRAME.
- ✓ `delete-other-windows` Make WINDOW fill its frame.
- · `delete-other-windows-internal` Make WINDOW fill its frame.
- · `delete-other-windows-vertically` Delete the windows in the same column with WINDOW, but not WINDOW itself.
- · `delete-pair` Delete a pair of characters enclosing ARG sexps that follow point.
- · `delete-process` Delete PROCESS: kill it and forget about it immediately.
- ✓ `delete-rectangle` Delete (don't save) text in the region-rectangle.
- ✓ `delete-region` Delete the text between START and END.
- · `delete-selection-local-mode` Toggle `delete-selection-mode' only in this buffer.
- · `delete-selection-mode` Toggle Delete Selection mode.
- ✓ `delete-trailing-whitespace` Delete trailing whitespace between START and END.
- · `delete-trailing-whitespace-mode` Delete trailing whitespace before saving the current buffer.
- ✓ `delete-whitespace-rectangle` Delete all whitespace following a specified column in each line.
- ✓ `delete-window` Delete specified WINDOW.
- · `delete-windows-on` Delete all windows showing BUFFER-OR-NAME.

## delimit

- · `delimit-columns-customize` Customize the `columns' group.
- · `delimit-columns-rectangle` Prettify all columns in a text rectangle.
- · `delimit-columns-region` Prettify all columns in a text region.

## delphi

- · `delphi-mode` Major mode for editing OPascal code.

## denato

- · `denato-region` Convert NATO spelling alphabet text in region to plain text.

## describe

- ✓ `describe-bindings` Display a buffer showing a list of all defined keys, and their definitions.
- · `describe-buffer-case-table` Describe the case table of the current buffer.
- · `describe-categories` Describe the category specifications in the current category table.
- · `describe-char` Describe position POS (interactively, point) and the char after POS.
- · `describe-character-set` Display information about built-in character set CHARSET.
- · `describe-char-fold-equivalences` Display characters equivalent to CHAR under character-folding.
- · `describe-coding-system` Display information about CODING-SYSTEM.
- ✓ `describe-command` Display the full documentation of COMMAND (a symbol).
- · `describe-copying` Display info on how you may redistribute copies of GNU Emacs.
- · `describe-current-coding-system` Display coding systems currently used, in detail.
- · `describe-current-coding-system-briefly` Display coding systems currently used in a brief format in echo area.
- · `describe-current-display-table` Describe the display table in use in the selected window and buffer.
- · `describe-distribution` Display info on how to obtain the latest version of GNU Emacs.
- · `describe-editorconfig-properties` Display EditorConfig properties extracted for current buffer.
- · `describe-face` Display the properties of face FACE on FRAME.
- · `describe-font` Display information about a font whose name is FONTNAME.
- · `describe-fontset` Display information about FONTSET.
- ✓ `describe-function` Display the full documentation of FUNCTION (a symbol).
- · `describe-gnu-project` Browse online information on the GNU project.
- · `describe-icon` Pop to a buffer to describe ICON.
- · `describe-input-method` Describe input method INPUT-METHOD.
- ✓ `describe-key` Display documentation of the function invoked by KEY-LIST.
- ✓ `describe-key-briefly` Print the name of the functions KEY-LIST invokes.
- · `describe-keymap` Describe key bindings in KEYMAP.
- · `describe-language-environment` Describe how Emacs supports language environment LANGUAGE-NAME.
- · `describe-minor-mode` Display documentation of a minor mode given as MINOR-MODE.
- · `describe-minor-mode-from-indicator` Display documentation of a minor mode specified by INDICATOR.
- · `describe-minor-mode-from-symbol` Display documentation of a minor mode given as a symbol, SYMBOL.
- ✓ `describe-mode` Display documentation of current major mode and minor modes.
- · `describe-no-warranty` Display info on all the kinds of warranty Emacs does NOT have.
- ✓ `describe-package` Display the full documentation of PACKAGE (a symbol).
- · `describe-personal-keybindings` Display all the personal keybindings defined by `bind-key'.
- · `describe-prefix-bindings` Describe the bindings of the prefix used to reach this command.
- · `describe-specified-language-support` Describe how Emacs supports the specified language environment.
- ✓ `describe-symbol` Display the full documentation of SYMBOL.
- · `describe-syntax` Describe the syntax specifications in the syntax table of BUFFER.
- · `describe-text-properties` Describe widgets, buttons, overlays, and text properties at POS.
- · `describe-theme` Display a description of the Custom theme THEME (a symbol).
- ✓ `describe-variable` Display the full documentation of VARIABLE (a symbol).
- · `describe-widget` Display a buffer with information about a widget.

## desktop

- · `desktop-change-dir` Change to desktop saved in DIRNAME.
- · `desktop-clear` Empty the Desktop.
- · `desktop-read` Read and process the desktop file in directory DIRNAME.
- · `desktop-remove` Delete desktop file in `desktop-dirname'.
- · `desktop-revert` Revert to the last loaded desktop.
- · `desktop-save` Save the state of Emacs in a desktop file in directory DIRNAME.
- · `desktop-save-in-desktop-dir` Save the desktop in directory `desktop-dirname'.
- · `desktop-save-mode` Toggle desktop saving (Desktop Save mode).

## diary

- · `diary` Generate the diary window for ARG days starting with the current date.
- · `diary-icalendar-export-file` Export DIARY-FILENAME to iCalendar format in FILENAME.
- · `diary-icalendar-export-region` Export diary entries between BEGIN and END to iCalendar format in FILENAME.
- · `diary-icalendar-import-buffer` Import iCalendar events from current buffer into diary.
- · `diary-icalendar-import-file` Import iCalendar diary entries from FILENAME into DIARY-FILENAME.
- · `diary-mail-entries` Send a mail message showing diary entries for next NDAYS days.
- · `diary-mode` Major mode for editing the diary file.

## dictionary

- · `dictionary` Create a new dictionary buffer and install `dictionary-mode'.
- · `dictionary-lookup-definition` Unconditionally lookup the word at point.
- · `dictionary-match-words` Search PATTERN in current default dictionary using default strategy.
- · `dictionary-mode` Mode for searching a dictionary.
- · `dictionary-mouse-popup-matching-words` Display entries matching the word at the cursor retrieved using EVENT.
- · `dictionary-popup-matching-words` Display entries matching WORD or the current word if not given.
- · `dictionary-search` Search for WORD in all the known dictionaries.
- · `dictionary-tooltip-mode` Display tooltips for the current word.

## diff

- · `diff` Find and display the differences between OLD and NEW files.
- · `diff-backup` Diff this file with its backup file or vice versa.
- · `diff-buffers` Find and display the differences between OLD and NEW buffers.
- · `diff-buffer-with-file` View the differences between BUFFER and its associated file.
- · `diff-minor-mode` Toggle Diff minor mode.
- · `diff-mode` Major mode for viewing/editing context diffs.

## dig

- · `dig` Query addresses of a DOMAIN using dig.

## digit

- ✓ `digit-argument` Part of the numeric argument for the next command.

## dired

- ✓ `dired` "Edit" directory DIRNAME--delete, rename, print, etc. some files in it.
- · `dired-at-point` Start Dired, defaulting to file at point.  See `ffap'.
- · `dired-from-menubar` Edit an existing directory.
- ✓ `dired-jump` Jump to Dired buffer corresponding to current buffer.
- ✓ `dired-jump-other-window` Like \[dired-jump] (`dired-jump') but in other window.
- · `dired-other-frame` "Edit" directory DIRNAME.  Like `dired' but make a new frame.
- · `dired-other-tab` "Edit" directory DIRNAME.  Like `dired' but make a new tab.
- ✓ `dired-other-window` "Edit" directory DIRNAME.  Like `dired' but select in another window.

## dirtrack

- · `dirtrack-mode` Toggle directory tracking in shell buffers (Dirtrack mode).

## disable

- · `disable-command` Require special confirmation to execute COMMAND from now on.
- · `disable-theme` Disable all variable and face settings defined by THEME.

## disassemble

- · `disassemble` Print disassembled code for OBJECT in (optional) BUFFER.

## display

- · `display-about-screen` Display the *About GNU Emacs* buffer.
- · `display-battery-mode` Toggle battery status display in mode line (Display Battery mode).
- · `display-buffer` Display BUFFER-OR-NAME in some window, without selecting it.
- · `display-buffer-other-frame` Display buffer BUFFER preferably in another frame.
- · `display-call-tree` Display a call graph of a specified file.
- · `display-fill-column-indicator-mode` Toggle display of `fill-column' indicator.
- ✓ `display-line-numbers-mode` Toggle display of line numbers in the buffer.
- ✓ `display-local-help` Display in the echo area `kbd-help' or `help-echo' text at point.
- · `display-time` Enable display of time, load level, and mail flag in mode lines.
- · `display-time-mode` Toggle display of time, load level, and mail flag in mode lines.
- · `display-time-world` Display a world clock buffer with times in various time zones.

## dissociated

- · `dissociated-press` Dissociate the text of the current buffer.

## dns

- · `dns-lookup-host` Look up the DNS information for HOST (name or IP address).
- · `dns-mode` Major mode for viewing and editing DNS master files.
- · `dns-mode-soa-increment-serial` Locate SOA record and increment the serial field.

## do

- · `do-auto-save` Auto-save all buffers that need it.

## doc

- · `doc-file-to-info` Produce a texinfo buffer with sorted doc-strings from the DOC file.
- · `doc-file-to-man` Produce an nroff buffer containing the doc-strings from the DOC file.
- · `doc-view-minor-mode` Toggle displaying buffer via Doc View (Doc View minor mode).
- · `doc-view-mode` Major mode in DocView buffers.

## dockerfile

- · `dockerfile-ts-mode` Major mode for editing Dockerfiles, powered by tree-sitter.

## doctex

- · `doctex-mode` Major mode to edit DocTeX files.

## doctor

- · `doctor` Switch to *doctor* buffer and start giving psychotherapy.

## double

- · `double-mode` Toggle special insertion on double keypresses (Double mode).

## down

- ✓ `down-list` Move forward down one level of parentheses.

## downcase

- · `downcase-dwim` Downcase words in the region, if active; if not, downcase word at point.
- ✓ `downcase-region` Convert the region to lower case.  In programs, wants two arguments.
- ✓ `downcase-word` Convert to lower case from point to end of word, moving over.

## dsssl

- · `dsssl-mode` Major mode for editing DSSSL code.

## dunnet

- · `dunnet` Switch to *dungeon* buffer and start game.

## duplicate

- · `duplicate-dwim` Duplicate the current line or region N times.
- · `duplicate-line` Duplicate the current line N times.

## dynamic

- · `dynamic-completion-mode` Toggle dynamic word-completion on or off.
- · `dynamic-setting-handle-config-changed-event` Handle config-changed-event on the display in EVENT.

## easy

- · `easy-menu-add` Ignore ARGUMENTS, do nothing, and return nil.
- · `easy-menu-remove` Ignore ARGUMENTS, do nothing, and return nil.

## ebnf

- · `ebnf-apply-style` Set STYLE as the current style.
- · `ebnf-customize` Customization for ebnf group.
- · `ebnf-delete-style` Delete style NAME.
- · `ebnf-despool` Send the spooled PostScript to the printer.
- · `ebnf-eps-buffer` Generate a PostScript syntactic chart image of the buffer in an EPS file.
- · `ebnf-eps-directory` Generate EPS files from EBNF files in DIRECTORY.
- · `ebnf-eps-file` Generate an EPS file from EBNF file FILE.
- · `ebnf-eps-region` Generate a PostScript syntactic chart image of the region in an EPS file.
- · `ebnf-find-style` Return style definition if NAME is already defined; otherwise, return nil.
- · `ebnf-insert-style` Insert a new style NAME with inheritance INHERITS and values VALUES.
- · `ebnf-merge-style` Merge values of style NAME with style VALUES.
- · `ebnf-pop-style` Pop a style from the stack of pushed styles and set it as the current style.
- · `ebnf-print-buffer` Generate and print a PostScript syntactic chart image of the buffer.
- · `ebnf-print-directory` Generate and print a PostScript syntactic chart image of DIRECTORY.
- · `ebnf-print-file` Generate and print a PostScript syntactic chart image of the file FILE.
- · `ebnf-print-region` Generate and print a PostScript syntactic chart image of the region.
- · `ebnf-push-style` Push the current style onto a stack and set STYLE as the current style.
- · `ebnf-reset-style` Reset current style.
- · `ebnf-spool-buffer` Generate and spool a PostScript syntactic chart image of the buffer.
- · `ebnf-spool-directory` Generate and spool a PostScript syntactic chart image of DIRECTORY.
- · `ebnf-spool-file` Generate and spool a PostScript syntactic chart image of the file FILE.
- · `ebnf-spool-region` Generate a PostScript syntactic chart image of the region and spool locally.
- · `ebnf-syntax-buffer` Do a syntactic analysis of the current buffer.
- · `ebnf-syntax-directory` Do a syntactic analysis of the files in DIRECTORY.
- · `ebnf-syntax-file` Do a syntactic analysis of the named FILE.
- · `ebnf-syntax-region` Do a syntactic analysis of a region.

## ebrowse

- · `ebrowse-back-in-position-stack` Move backward in the position stack.
- · `ebrowse-electric-choose-tree` Return a buffer containing a tree or nil if no tree found or canceled.
- · `ebrowse-electric-position-menu` List positions in the position stack in an electric buffer.
- · `ebrowse-forward-in-position-stack` Move forward in the position stack.
- · `ebrowse-member-mode` Major mode for Ebrowse member buffers.
- · `ebrowse-save-tree` Save current tree in same file it was loaded from.
- · `ebrowse-save-tree-as` Write the current tree data structure to a file.
- · `ebrowse-statistics` Display statistics for a class tree.
- · `ebrowse-tags-complete-symbol` Perform completion on the C++ symbol preceding point.
- · `ebrowse-tags-find-declaration` Find declaration of member at point.
- · `ebrowse-tags-find-declaration-other-frame` Find definition of member at point in other frame.
- · `ebrowse-tags-find-declaration-other-window` Find declaration of member at point in other window.
- · `ebrowse-tags-find-definition` Find definition of member at point.
- · `ebrowse-tags-find-definition-other-frame` Find definition of member at point in other frame.
- · `ebrowse-tags-find-definition-other-window` Find definition of member at point in other window.
- · `ebrowse-tags-loop-continue` Repeat last operation on files in tree.
- · `ebrowse-tags-query-replace` Query replace FROM with TO in all files of a class tree.
- · `ebrowse-tags-search` Search for REGEXP in all files in a tree.
- · `ebrowse-tags-search-member-use` Search for call sites of a member.
- · `ebrowse-tags-view-declaration` View declaration of member at point.
- · `ebrowse-tags-view-definition` View definition of member at point.
- · `ebrowse-tags-view-definition-other-frame` View definition of member at point in other frame.
- · `ebrowse-tags-view-definition-other-window` View definition of member at point in other window.
- · `ebrowse-tree-mode` Major mode for Ebrowse class tree buffers.

## ebuffers

- · `ebuffers` Run Ediff on a pair of buffers, BUFFER-A and BUFFER-B.

## ebuffers3

- · `ebuffers3` Run Ediff on three buffers, BUFFER-A, BUFFER-B, and BUFFER-C.

## edebug

- · `edebug-all-defs` Toggle edebugging of all definitions.
- · `edebug-all-forms` Toggle edebugging of all forms.
- · `edebug-defun` Evaluate the top level form point is in, stepping through with Edebug.
- · `edebug-eval-top-level-form` Evaluate the top level form point is in, stepping through with Edebug.

## ediff

- · `ediff` Run Ediff on a pair of files, FILE-A and FILE-B.
- · `ediff-backup` Run Ediff on FILE and its backup file.
- · `ediff-buffers` Run Ediff on a pair of buffers, BUFFER-A and BUFFER-B.
- · `ediff-buffers3` Run Ediff on three buffers, BUFFER-A, BUFFER-B, and BUFFER-C.
- · `ediff-current-file` Start ediff between current buffer and its file on disk.
- · `ediff-customize` 
- · `ediff-directories` Run Ediff on directories DIR1 and DIR2, comparing files.
- · `ediff-directories3` Run Ediff on directories DIR1, DIR2, and DIR3, comparing files.
- · `ediff-directory-revisions` Run Ediff on a directory, DIR1, comparing its files with their revisions.
- · `ediff-documentation` Display Ediff's manual.
- · `ediff-files` Run Ediff on a pair of files, FILE-A and FILE-B.
- · `ediff-files3` Run Ediff on three files, FILE-A, FILE-B, and FILE-C.
- · `ediff-merge` Merge two files without ancestor.
- · `ediff-merge-buffers` Merge buffers without ancestor.
- · `ediff-merge-buffers-with-ancestor` Merge buffers with ancestor.
- · `ediff-merge-directories` Run Ediff on a pair of directories, DIR1 and DIR2, merging files that have
- · `ediff-merge-directories-with-ancestor` Merge files in DIR1 and DIR2 using files in ANCESTOR-DIR as ancestors.
- · `ediff-merge-directory-revisions` Run Ediff on a directory, DIR1, merging its files with their revisions.
- · `ediff-merge-directory-revisions-with-ancestor` Run Ediff on DIR1 and merge its files with their revisions and ancestors.
- · `ediff-merge-files` Merge two files without ancestor.
- · `ediff-merge-files-with-ancestor` Merge two files with ancestor.
- · `ediff-merge-revisions` Run Ediff by merging two revisions of a file.
- · `ediff-merge-revisions-with-ancestor` Run Ediff by merging two revisions of a file with a common ancestor.
- · `ediff-merge-with-ancestor` Merge two files with ancestor.
- · `ediff-patch-buffer` Run Ediff by patching the buffer specified at prompt.
- · `ediff-patch-file` Query for a file name, and then run Ediff by patching that file.
- · `ediff-regions-linewise` Run Ediff on a pair of regions in specified buffers.
- · `ediff-regions-wordwise` Run Ediff on a pair of regions in specified buffers.
- · `ediff-revision` Run Ediff by comparing versions of a file.
- · `ediff-show-registry` Display Ediff's registry.
- · `ediff-toggle-multiframe` Switch from multiframe display to single-frame display and back.
- · `ediff-toggle-use-toolbar` Enable or disable Ediff toolbar.
- · `ediff-version` Return string describing the version of Ediff.
- · `ediff-windows-linewise` Compare WIND-A and WIND-B, which are selected by clicking, linewise.
- · `ediff-windows-wordwise` Compare WIND-A and WIND-B, which are selected by clicking, wordwise.

## ediff3

- · `ediff3` Run Ediff on three files, FILE-A, FILE-B, and FILE-C.

## edir

- · `edir-merge-revisions` Run Ediff on a directory, DIR1, merging its files with their revisions.
- · `edir-merge-revisions-with-ancestor` Run Ediff on DIR1 and merge its files with their revisions and ancestors.
- · `edir-revisions` Run Ediff on a directory, DIR1, comparing its files with their revisions.

## edirs

- · `edirs` Run Ediff on directories DIR1 and DIR2, comparing files.
- · `edirs-merge` Run Ediff on a pair of directories, DIR1 and DIR2, merging files that have
- · `edirs-merge-with-ancestor` Merge files in DIR1 and DIR2 using files in ANCESTOR-DIR as ancestors.

## edirs3

- · `edirs3` Run Ediff on directories DIR1, DIR2, and DIR3, comparing files.

## edit

- · `edit-abbrevs` Alter abbrev definitions by editing the list of abbrevs.
- · `edit-abbrevs-redefine` Redefine abbrevs according to current buffer contents.
- · `edit-bookmarks` Display a list of existing bookmarks.
- · `edit-kbd-macro` Edit a keyboard macro.
- · `edit-last-kbd-macro` Edit the most recently defined keyboard macro.
- · `edit-named-kbd-macro` Edit a keyboard macro which has been given a name by `name-last-kbd-macro'.
- · `edit-picture` Switch to Picture mode, in which a quarter-plane screen model is used.
- · `edit-tab-stops` Edit the tab stops used by `tab-to-tab-stop'.
- · `edit-tab-stops-note-changes` Put edited tab stops into effect.

## editorconfig

- · `editorconfig-apply` Get and apply EditorConfig properties to current buffer.
- · `editorconfig-conf-mode` Major mode for editing .editorconfig files.
- · `editorconfig-display-current-properties` Display EditorConfig properties extracted for current buffer.
- · `editorconfig-find-current-editorconfig` Find the closest .editorconfig file for current file.
- · `editorconfig-mode` Toggle EditorConfig feature.

## edt

- · `edt-emulation-on` Turn on EDT Emulation.
- · `edt-set-scroll-margins` Set scroll margins.

## eglot

- · `eglot` Start LSP server for PROJECT's buffers under MANAGED-MAJOR-MODES.
- · `eglot-manual` Read Eglot's manual.
- · `eglot-update` Update Eglot to latest version.
- · `eglot-upgrade-eglot` Update Eglot to latest version.

## ehelp

- · `ehelp-command` Prefix command for ehelp.

## eieio

- · `eieio-browse` Create an object browser window to show all objects.

## eldoc

- · `eldoc` Document thing at point.
- ✓ `eldoc-doc-buffer` Get or display ElDoc documentation buffer.
- · `eldoc-mode` Toggle echo area display of Lisp objects at point (ElDoc mode).
- · `eldoc-print-current-symbol-info` Document thing at point.

## electric

- · `electric-buffer-list` Pop up the Buffer Menu in an "electric" window.
- · `electric-indent-just-newline` Insert just a newline, without any auto-indentation.
- · `electric-indent-local-mode` Toggle `electric-indent-mode' only in this buffer.
- · `electric-indent-mode` Toggle on-the-fly reindentation of text lines (Electric Indent mode).
- · `electric-layout-local-mode` Toggle `electric-layout-mode' only in this buffer.
- · `electric-layout-mode` Automatically insert newlines around some chars.
- · `electric-newline-and-maybe-indent` Insert a newline.
- · `electric-pair-local-mode` Toggle `electric-pair-mode' only in this buffer.
- · `electric-pair-mode` Toggle automatic pairing of delimiters (Electric Pair mode).
- · `electric-quote-local-mode` Toggle `electric-quote-mode' only in this buffer.
- · `electric-quote-mode` Toggle on-the-fly requoting (Electric Quote mode).

## elide

- · `elide-head` Hide header material in buffer according to `elide-head-headers-to-hide'.
- · `elide-head-mode` Toggle eliding (hiding) header material in the current buffer.

## elisp

- · `elisp-byte-code-mode` Major mode for *.elc files.
- · `elisp-byte-compile-buffer` Byte compile the current buffer, but don't write a file.
- · `elisp-byte-compile-file` Byte compile the file the current buffer is visiting.
- · `elisp-enable-lexical-binding` Make the current buffer use `lexical-binding'.
- ✓ `elisp-eval-region-or-buffer` Evaluate the forms in the active region or the whole current buffer.
- · `elisp-flymake--batch-compile-for-flymake` Helper for `elisp-flymake-byte-compile'.
- · `elisp-index-search` Look up TOPIC in the indices of the Emacs Lisp Reference Manual.
- · `elisp-last-sexp-toggle-display` Toggle between abbreviated and unabbreviated printed representations.
- · `elisp-scope-describe-symbol-role` Describe ROLE of a symbol.

## elixir

- · `elixir-ts-mode` Major mode for editing Elixir, powered by tree-sitter.

## elp

- · `elp-instrument-function` Instrument FUNSYM for profiling.
- · `elp-instrument-list` Instrument, for profiling, all functions in `elp-function-list'.
- · `elp-instrument-package` Instrument for profiling, all functions which start with PREFIX.
- · `elp-results` Display current profiling results.

## emacs

- · `emacs-authors-mode` Major mode for viewing "etc/AUTHORS" from the Emacs distribution.
- · `emacs-build-description` Insert a description of the current Emacs build in the current buffer.
- · `emacs-index-search` Look up TOPIC in the indices of the Emacs User Manual.
- · `emacs-init-time` Return a string giving the duration of the Emacs initialization.
- · `emacs-lisp-byte-compile` Byte-compile the current buffer's file.
- · `emacs-lisp-byte-compile-and-load` Byte-compile the current file (if it has changed), then load compiled code.
- · `emacs-lisp-macroexpand` Macroexpand the form after point.
- · `emacs-lisp-mode` Major mode for editing Lisp code to run in Emacs.
- · `emacs-lisp-mode-menu` Menu for Emacs Lisp mode.
- · `emacs-lisp-native-compile` Native-compile the current buffer's file (if it has changed).
- · `emacs-lisp-native-compile-and-load` Native-compile the current buffer's file (if it has changed), then load it.
- · `emacs-lock-mode` Toggle Emacs Lock mode in the current buffer.
- · `emacs-news-mode` Major mode for editing the Emacs NEWS file.
- · `emacs-news-view-mode` Major mode for viewing the Emacs NEWS file.
- · `emacs-uptime` Return a string giving the uptime of this instance of Emacs.
- · `emacs-version` Display the version of Emacs that is running in this session.

## emerge

- · `emerge-buffers` Run Emerge on two buffers BUFFER-A and BUFFER-B.
- · `emerge-buffers-with-ancestor` Run Emerge on two buffers, giving another buffer as the ancestor.
- · `emerge-files` Run Emerge on two files FILE-A and FILE-B.
- · `emerge-files-with-ancestor` Run Emerge on two files, giving another file as the ancestor.
- · `emerge-merge-directories` 
- · `emerge-revisions` Emerge two RCS revisions of a file.
- · `emerge-revisions-with-ancestor` Emerge two RCS revisions of a file, with another revision as ancestor.

## emoji

- · `emoji-describe` Display the name of the grapheme cluster composed from GLYPH.
- · `emoji-insert` 
- · `emoji-list` List Emoji and allow selecting and inserting one of them.
- · `emoji-list-select` 
- · `emoji-recent` 
- · `emoji-search` 
- · `emoji-zoom-decrease` Decrease the size of the character under point.
- · `emoji-zoom-increase` Increase the size of the character under point.
- · `emoji-zoom-reset` Reset the size of the character under point.

## enable

- · `enable-command` Allow COMMAND to be executed without special confirmation from now on.
- · `enable-flow-control` Toggle flow control handling.
- · `enable-theme` Reenable all variable and face settings defined by THEME.

## encode

- · `encode-coding-region` Encode the current region using the specified coding system.
- · `encode-hz-buffer` Encode the text in the current buffer to HZ.
- · `encode-hz-region` Encode the text in the current region to HZ.

## end

- · `end-kbd-macro` Finish defining a keyboard macro.
- ✓ `end-of-buffer` Move point to the end of the buffer.
- ✓ `end-of-buffer-other-window` Move point to the end of the buffer in the other window.
- ✓ `end-of-defun` Move forward to next end of defun.
- ✓ `end-of-line` Move point to end of current line (in the logical order).
- · `end-of-visual-line` Move point to end of current visual line.

## enlarge

- ✓ `enlarge-window` Make the selected window DELTA lines taller.
- ✓ `enlarge-window-horizontally` Make selected window DELTA columns wider.

## enriched

- · `enriched-mode` Minor mode for editing text/enriched files.

## ensure

- · `ensure-empty-lines` Ensure that there are LINES number of empty lines before point.

## epa

- · `epa-decrypt-armor-in-region` Decrypt OpenPGP armors in the current region between START and END.
- · `epa-decrypt-file` Decrypt DECRYPT-FILE into PLAIN-FILE.
- · `epa-decrypt-region` Decrypt the current region between START and END.
- · `epa-delete-keys` Delete selected KEYS.
- · `epa-dired-do-decrypt` Decrypt marked files.
- · `epa-dired-do-encrypt` Encrypt marked files.
- · `epa-dired-do-sign` Sign marked files.
- · `epa-dired-do-verify` Verify marked files.
- · `epa-encrypt-file` Encrypt FILE for RECIPIENTS.
- · `epa-encrypt-region` Encrypt the current region between START and END for RECIPIENTS.
- · `epa-export-keys` Export selected KEYS to FILE.
- · `epa-file-disable` 
- · `epa-file-enable` 
- · `epa-file-name-regexp-update` Update `file-name-handler-alist' after configuring outside Custom.
- · `epa-global-mail-mode` Minor mode to hook EasyPG into Mail mode.
- · `epa-import-armor-in-region` Import keys in the OpenPGP armor format in the current region from START to END.
- · `epa-import-keys` Import keys from FILE.
- · `epa-import-keys-region` Import keys from the region.
- · `epa-insert-keys` Insert selected KEYS after the point.
- · `epa-list-keys` List all keys matched with NAME from the public keyring.
- · `epa-list-secret-keys` List all keys matched with NAME from the private keyring.
- · `epa-mail-decrypt` Decrypt OpenPGP armors in the current buffer.
- · `epa-mail-encrypt` Encrypt the outgoing mail message in the current buffer.
- · `epa-mail-import-keys` Import keys in the OpenPGP armor format in the current buffer.
- · `epa-mail-mode` A minor-mode for composing encrypted/clearsigned mails.
- · `epa-mail-sign` Sign the current buffer.
- · `epa-mail-verify` Verify OpenPGP cleartext signed messages in the current buffer.
- · `epa-search-keys` Ask a keyserver for all keys matching QUERY.
- · `epa-sign-file` Sign FILE by SIGNERS keys selected.
- · `epa-sign-region` Sign the current region between START and END by SIGNERS keys selected.
- · `epa-verify-cleartext-in-region` Verify OpenPGP cleartext signed messages in current region from START to END.
- · `epa-verify-file` Verify FILE.
- · `epa-verify-region` Verify the current region between START and END.

## epatch

- · `epatch` Query for a file name, and then run Ediff by patching that file.
- · `epatch-buffer` Run Ediff by patching the buffer specified at prompt.

## erase

- · `erase-buffer` Delete the entire contents of the current buffer.

## erc

- · `erc` Connect to an Internet Relay Chat SERVER on a non-TLS PORT.
- · `erc-select` Connect to an Internet Relay Chat SERVER on a non-TLS PORT.
- · `erc-server-select` Interactively connect to a server from `erc-server-alist'.
- · `erc-tls` Connect to an IRC server over a TLS-encrypted connection.

## eregistry

- · `eregistry` Display Ediff's registry.

## erevision

- · `erevision` Run Ediff by comparing versions of a file.

## ert

- · `ert` Run the tests specified by SELECTOR and display the results in a buffer.
- · `ert-describe-test` Display the documentation for TEST-OR-TEST-NAME (a symbol or ert-test).
- · `ert-run-tests-interactively` Run the tests specified by SELECTOR and display the results in a buffer.

## erts

- · `erts-mode` Major mode for editing erts (Emacs testing) files.

## eshell

- · `eshell` Create an interactive Eshell buffer.
- · `eshell-command` Execute the Eshell command string COMMAND.
- · `eshell-execute-file` Execute a series of Eshell commands in FILE, passing ARGS.
- · `eshell-mode` Emacs shell interactive mode.

## etags

- · `etags-regen-mode` Minor mode to automatically generate and update tags tables.

## etc

- · `etc-authors-mode` Major mode for viewing "etc/AUTHORS" from the Emacs distribution.

## ethio

- · `ethio-fidel-to-sera-buffer` Convert all the FIDEL characters in the current buffer to the SERA format.
- · `ethio-fidel-to-sera-marker` Convert the regions surrounded by "<sera>" and "</sera>" from FIDEL to SERA.
- · `ethio-fidel-to-sera-region` Convert all the FIDEL characters in the region to the SERA format.
- · `ethio-fidel-to-tex-buffer` Convert each FIDEL characters in the current buffer into a fidel-tex command.
- · `ethio-input-special-character` This function is deprecated.
- · `ethio-insert-ethio-space` Insert the Ethiopic word delimiter (the colon-like character).
- · `ethio-modify-vowel` Modify the vowel of the FIDEL that is under the cursor.
- · `ethio-replace-space` Replace ASCII spaces with Ethiopic word separators in the region.
- · `ethio-sera-to-fidel-buffer` Convert the current buffer from SERA to FIDEL.
- · `ethio-sera-to-fidel-marker` Convert regions surrounded by "<sera>" and "</sera>" from SERA to FIDEL.
- · `ethio-sera-to-fidel-region` Convert the characters in region from SERA to FIDEL.
- · `ethio-tex-to-fidel-buffer` Convert fidel-tex commands in the current buffer into FIDEL chars.

## eudc

- · `eudc-edit-hotlist` Edit the hotlist of directory servers in a specialized buffer.
- · `eudc-expand-inline` Query the directory server, and expand the query string before point.
- · `eudc-expand-try-all` Wrap `eudc-expand-inline' with a prefix argument.
- · `eudc-get-email` Get the email field of NAME from the directory server.
- · `eudc-get-phone` Get the phone field of NAME from the directory server.
- · `eudc-insert-record-at-point-into-bbdb` Insert record at point into the BBDB database.
- · `eudc-load-eudc` Load the Emacs Unified Directory Client.
- · `eudc-query-form` Display a form to query the directory server.
- · `eudc-set-server` Set the directory server to SERVER using PROTOCOL.
- · `eudc-try-bbdb-insert` Call `eudc-insert-record-at-point-into-bbdb' if on a record.

## eval

- · `eval-buffer` Execute the accessible portion of current buffer as Lisp code.
- ✓ `eval-defun` Evaluate top-level form around point and instrument it if EDEBUG-IT is non-nil.
- ✓ `eval-expression` Evaluate EXP and print value in the echo area.
- ✓ `eval-last-sexp` Evaluate sexp before point; print value in the echo area.
- ✓ `eval-print-last-sexp` Evaluate sexp before point; print value into current buffer.
- · `eval-region` Execute the region as Lisp code.

## eww

- · `eww` Fetch URL and render the page.
- · `eww-browse` Function to be run to parse command line URLs.
- · `eww-list-bookmarks` Display the eww bookmarks.
- · `eww-open-file` Render FILE using EWW.
- ✓ `eww-search-words` Search the web for the text in the region.

## exchange

- ✓ `exchange-point-and-mark` Put the mark where point is now, and point where the mark is now.

## executable

- · `executable-interpret` Run script with user-specified args, and collect output in a buffer.
- · `executable-set-magic` Set this buffer's interpreter to INTERPRETER with optional ARGUMENT.

## execute

- ✓ `execute-extended-command` Read a command name, then read the arguments and call the command.
- · `execute-extended-command-cycle` Choose the next version of the extended command predicates.
- ✓ `execute-extended-command-for-buffer` Query user for a command relevant for the current mode, and then execute it.

## exit

- · `exit-minibuffer` Terminate this minibuffer argument.
- ✓ `exit-recursive-edit` Exit from the innermost recursive edit or minibuffer.
- · `exit-splash-screen` Stop displaying the splash screen buffer.

## expand

- · `expand-abbrev` Expand the abbrev before point, if there is an abbrev there.
- · `expand-jump-to-next-slot` Move the cursor to the next slot in the last abbrev expansion.
- · `expand-jump-to-previous-slot` Move the cursor to the previous slot in the last abbrev expansion.
- · `expand-mail-aliases` Expand all mail aliases in suitable header fields found between BEG and END.
- · `expand-region-abbrevs` For each abbrev occurrence in the region, offer to expand it.

## f90

- · `f90-mode` Major mode for editing Fortran 90,95 code in free format.

## faceup

- · `faceup-clean-buffer` Remove faceup markup from buffer.
- · `faceup-render-view-buffer` Convert BUFFER containing Faceup markup to a new buffer and display it.
- · `faceup-view-buffer` Display the faceup representation of the current buffer.
- · `faceup-write-file` Save the faceup representation of the current buffer to the file FILE-NAME.

## feedmail

- · `feedmail-queue-reminder` Perform some kind of reminder activity about queued and draft messages.
- · `feedmail-run-the-queue` Visit each message in the feedmail queue directory and send it out.
- · `feedmail-run-the-queue-global-prompt` Like `feedmail-run-the-queue', but with a global confirmation prompt.
- · `feedmail-run-the-queue-no-prompts` Like `feedmail-run-the-queue', but suppress confirmation prompts.

## fetch

- · `fetch-bytecode` Ignore ARGUMENTS, do nothing, and return nil.

## ff

- · `ff-find-other-file` Find the header or source file corresponding to this file.
- · `ff-find-other-file-other-window` Visit the file you point at in another window.
- · `ff-find-related-file` Find the header or source file corresponding to this file.
- · `ff-get-other-file` Find the header or source file corresponding to this file.
- · `ff-mouse-find-other-file` Find the header or source file corresponding to this file.
- · `ff-mouse-find-other-file-other-window` Visit the file you point at in another window.

## ffap

- · `ffap` Find FILENAME, guessing a default from text around point.
- · `ffap-at-mouse` Find file or URL guessed from text around mouse click.
- · `ffap-bindings` Evaluate the forms in variable `ffap-bindings'.
- · `ffap-menu` Put up a menu of files and URLs mentioned in this buffer.
- · `ffap-next` Search buffer for next file or URL, and run ffap.

## fido

- · `fido-mode` An enhanced `icomplete-mode' that emulates `ido-mode'.
- · `fido-vertical-mode` Toggle vertical candidate display in `fido-mode'.

## file

- · `file-cache-add-directory` Add all files in DIRECTORY to the file cache.
- · `file-cache-add-directory-list` Add DIRECTORIES (a list of directory names) to the file cache.
- · `file-cache-add-directory-recursively` Add DIR and any subdirectories to the file-cache.
- · `file-cache-add-directory-using-find` Use the `find' command to add files to the file cache.
- · `file-cache-add-directory-using-locate` Use the `locate' command to add files to the file cache.
- · `file-cache-add-file` Add FILE to the file cache.
- · `file-cache-minibuffer-complete` Complete a filename in the minibuffer using a preloaded cache.
- · `file-name-shadow-mode` Toggle file-name shadowing in minibuffers (File-Name Shadow mode).
- · `file-notify-handle-event` Handle a file system monitoring event, coming from backends.
- · `file-to-register` Insert FILE-NAME into REGISTER.

## fill

- · `fill-individual-paragraphs` Fill paragraphs of uniform indentation within the region.
- · `fill-nonuniform-paragraphs` Fill paragraphs within the region, allowing varying indentation within each.
- ✓ `fill-paragraph` Fill paragraph at or after point.
- · `fill-paragraph-semlf` Fill paragraph at or after point using semantic linefeeds.
- · `fill-region` Fill each of the paragraphs in the region.
- · `fill-region-as-paragraph` Fill the region as if it were a single paragraph.
- · `fill-region-as-paragraph-default` Fill the region as if it were a single paragraph.
- · `fill-region-as-paragraph-semlf` Fill the region using semantic linefeeds as if it were a single paragraph.

## find

- ✓ `find-alternate-file` Find file FILENAME, select its buffer, kill previous buffer.
- · `find-alternate-file-other-window` Find file FILENAME as a replacement for the file in the next window.
- · `find-dired` Run `find' and go into Dired mode on a buffer of the output.
- · `find-dired-with-command` Run `find' and go into Dired mode on a buffer of the output.
- · `find-face-definition` Find the definition of FACE.  FACE defaults to the name near point.
- ✓ `find-file` Edit file FILENAME.
- · `find-file-at-point` Find FILENAME, guessing a default from text around point.
- · `find-file-existing` Edit the existing file FILENAME.
- · `find-file-literally` Visit file FILENAME with no conversion of any kind.
- · `find-file-other-frame` Edit file FILENAME, in another frame.
- · `find-file-other-tab` Edit file FILENAME, in another tab.
- ✓ `find-file-other-window` Edit file FILENAME, in another window.
- ✓ `find-file-read-only` Edit file FILENAME but don't allow changes.
- · `find-file-read-only-other-frame` Edit file FILENAME in another frame but don't allow changes.
- · `find-file-read-only-other-tab` Edit file FILENAME, in another tab, but don't allow changes.
- ✓ `find-file-read-only-other-window` Edit file FILENAME in another window but don't allow changes.
- · `find-function` Find the definition of the Emacs Lisp FUNCTION near point.
- · `find-function-at-point` Find directly the function at point in the other window.
- · `find-function-mode` Enable some key bindings for the `find-function' family of functions.
- · `find-function-on-key` Find the function that KEY invokes.  KEY is a string.
- · `find-function-on-key-other-frame` Find, in the other frame, the function that KEY invokes.
- · `find-function-on-key-other-window` Find, in the other window, the function that KEY invokes.
- · `find-function-other-frame` Find, in another frame, the definition of FUNCTION near point.
- · `find-function-other-window` Find, in another window, the definition of FUNCTION near point.
- · `find-grep` Run grep via find, with user-specified args COMMAND-ARGS.
- · `find-grep-dired` Find files in DIR that contain matches for REGEXP and start Dired on output.
- · `find-library` Find the Emacs Lisp source of LIBRARY.
- · `find-library-other-frame` Find the Emacs Lisp source of LIBRARY in another frame.
- · `find-library-other-window` Find the Emacs Lisp source of LIBRARY in another window.
- · `find-lisp-find-dired` Find the files within DIR whose names match REGEXP.
- · `find-lisp-find-dired-filter` Change the filter on a `find-lisp-find-dired' buffer to REGEXP.
- · `find-lisp-find-dired-subdirectories` Find all subdirectories of DIR.
- · `find-lisp-find-dired-subdirs-other-window` Same as `find-lisp-find-dired-subdirectories', but use another window.
- · `find-name-dired` Search DIR recursively for files matching the globbing PATTERN,
- · `find-sibling-file` Visit a "sibling" file of FILE.
- · `find-tag` Find tag (in current tags table) whose name contains TAGNAME.
- · `find-tag-noselect` Find tag (in current tags table) whose name contains TAGNAME.
- · `find-tag-other-frame` Find tag (in current tags table) whose name contains TAGNAME.
- · `find-tag-other-window` Find tag (in current tags table) whose name contains TAGNAME.
- · `find-tag-regexp` Find tag (in current tags table) whose name matches REGEXP.
- · `find-variable` Find the definition of the VARIABLE at or before point.
- · `find-variable-at-point` Find directly the variable at point in the other window.
- · `find-variable-other-frame` Find, in another frame, the definition of VARIABLE near point.
- · `find-variable-other-window` Find, in another window, the definition of VARIABLE near point.

## finder

- ✓ `finder-by-keyword` Find packages matching a given keyword.
- · `finder-commentary` Display FILE's commentary section.
- · `finder-list-keywords` Display descriptions of the keywords in the Finder buffer.

## finger

- · `finger` Finger USER on HOST.

## first

- · `first-completion` Move to the first item in the completions buffer.
- · `first-error` Restart at the first error.

## fit

- · `fit-frame-to-buffer` Adjust size of FRAME to display the contents of its buffer exactly.
- · `fit-window-to-buffer` Adjust size of WINDOW to display its buffer's contents exactly.

## fixup

- ✓ `fixup-whitespace` Fixup white space between objects around point.

## flush

- · `flush-lines` Delete lines containing matches for REGEXP.

## flymake

- · `flymake-mode` Toggle Flymake mode on or off.

## flyspell

- · `flyspell-buffer` Flyspell whole buffer.
- · `flyspell-mode` Toggle on-the-fly spell checking (Flyspell mode).
- · `flyspell-prog-mode` Turn on `flyspell-mode' for comments and strings.
- · `flyspell-region` Flyspell text between BEG and END.

## follow

- · `follow-delete-other-windows-and-split` Create two side by side windows and enter Follow mode.
- · `follow-mode` Toggle Follow mode.
- · `follow-scroll-down` Scroll text in a Follow mode window chain down.
- · `follow-scroll-down-window` Scroll text in a Follow mode window down by that window's size.
- · `follow-scroll-up` Scroll text in a Follow mode window chain up.
- · `follow-scroll-up-window` Scroll text in a Follow mode window up by that window's size.

## font

- · `font-lock-after-fontify-buffer` Ignore ARGUMENTS, do nothing, and return nil.
- · `font-lock-after-unfontify-buffer` Ignore ARGUMENTS, do nothing, and return nil.
- · `font-lock-debug-fontify` Reinitialize the font-lock machinery and (re-)fontify the buffer.
- · `font-lock-fontify-block` Fontify some lines the way `font-lock-fontify-buffer' would.
- · `font-lock-fontify-buffer` Fontify the current buffer the way the function `font-lock-mode' would.
- · `font-lock-mode` Toggle syntax highlighting in this buffer (Font Lock mode).
- ✓ `font-lock-update` Update the syntax highlighting in this buffer.
- · `font-show-log` Show log of font listing and opening.

## footnote

- · `footnote-mode` Toggle Footnote mode.

## format

- · `format-decode-buffer` Translate the buffer from some FORMAT.
- · `format-decode-region` Decode the region from some format.
- · `format-encode-buffer` Translate the buffer into FORMAT.
- · `format-encode-region` Translate the region into some FORMAT.
- · `format-find-file` Find the file FILENAME using data format FORMAT.
- · `format-insert-file` Insert the contents of file FILENAME using data format FORMAT.
- · `format-write-file` Write current buffer into FILENAME, using a format based on FORMAT.

## forms

- · `forms-find-file` Visit a file in Forms mode.
- · `forms-find-file-other-window` Visit a file in Forms mode in other window.
- · `forms-mode` Major mode to visit files in a field-structured manner using a form.

## fortran

- · `fortran-mode` Major mode for editing Fortran code in fixed format.

## fortune

- · `fortune` Display a fortune cookie.
- · `fortune-add-fortune` Add STRING to a fortune file FILE.
- · `fortune-compile` Compile fortune file.
- · `fortune-from-region` Append the current region to a local fortune-like data file.
- · `fortune-message` Display a fortune cookie to the mini-buffer.
- · `fortune-to-signature` Create signature from output of the fortune program.

## forward

- · `forward-button` Move to the Nth next button, or Nth previous button if N is negative.
- ✓ `forward-char` Move point N characters forward (backward if N is negative).
- · `forward-line` Move N lines forward (backward if N is negative).
- ✓ `forward-list` Move forward across one balanced group of parentheses.
- ✓ `forward-page` Move forward to page boundary.  With arg, repeat, or go back if negative.
- ✓ `forward-paragraph` Move forward to end of paragraph.
- · `forward-same-syntax` Move point past all characters with the same syntax class.
- ✓ `forward-sentence` Move forward to next end of sentence.  With argument ARG, repeat.
- ✓ `forward-sexp` Move forward across one balanced expression (sexp).
- · `forward-symbol` Move point to the next position that is the end of a symbol.
- · `forward-to-indentation` Move forward ARG lines and position at first nonblank character.
- · `forward-to-word` Move forward until encountering the beginning of the ARGth word.
- · `forward-whitespace` Move point to the end of the next sequence of whitespace chars.
- ✓ `forward-word` Move point forward ARG words (backward if ARG is negative).

## frame

- · `frame-configuration-to-register` Store the window configurations of all frames in REGISTER.

## frameset

- · `frameset-to-register` Store the current frameset in register REGISTER.

## fringe

- · `fringe-mode` Set the default appearance of fringes on all frames.

## ftp

- · `ftp` Run `ftp-program' to connect to HOST.

## full

- · `full-calc` Invoke the Calculator and give it a full-sized window.
- · `full-calc-keypad` Invoke the Calculator in full-screen "visual keypad" mode.

## fullwidth

- · `fullwidth-region` Convert ASCII characters in the region to their fullwidth variants.
- · `fullwidth-word` Convert fullwidth characters in word at point, moving over the word.

## fundamental

- · `fundamental-mode` Major mode not specialized for anything in particular.

## garbage

- · `garbage-collect` Reclaim storage for Lisp objects no longer needed.

## gdb

- · `gdb` Run gdb passing it COMMAND-LINE as arguments.
- · `gdb-enable-debug` Toggle logging of transaction between Emacs and Gdb.
- · `gdb-script-mode` Major mode for editing GDB scripts.

## generic

- · `generic-mode` Enter generic mode MODE.

## getenv

- · `getenv` Get the value of environment variable VARIABLE.

## glasses

- · `glasses-mode` Minor mode for making identifiers likeThis readable.

## global

- · `global-auto-composition-mode` Toggle Auto Composition mode in all buffers.
- · `global-auto-revert-mode` Toggle Global Auto-Revert Mode.
- · `global-completion-preview-mode` Toggle Completion-Preview mode in many buffers.
- · `global-cwarn-mode` Toggle Cwarn mode in many buffers.
- · `global-dictionary-tooltip-mode` Enable/disable `dictionary-tooltip-mode' for all buffers.
- · `global-display-fill-column-indicator-mode` Toggle Display-Fill-Column-Indicator mode in many buffers.
- · `global-display-line-numbers-mode` Toggle Display-Line-Numbers mode in many buffers.
- · `global-ede-mode` Toggle global EDE (Emacs Development Environment) mode.
- · `global-eldoc-mode` Toggle Eldoc mode in many buffers.
- · `global-font-lock-mode` Toggle Font-Lock mode in many buffers.
- · `global-goto-address-mode` Toggle Goto-Address mode in many buffers.
- · `global-highlight-changes-mode` Toggle Highlight-Changes mode in many buffers.
- · `global-hi-lock-mode` Toggle Hi-Lock mode in many buffers.
- · `global-hl-line-mode` Toggle line highlighting in all buffers (Global Hl-Line mode).
- · `global-prettify-symbols-mode` Toggle Prettify-Symbols mode in many buffers.
- · `global-reveal-mode` Toggle Reveal mode in all buffers (Global Reveal mode).
- · `global-set-key` Give KEY a global binding as COMMAND.
- · `global-so-long-mode` Toggle automated performance mitigation for files with long lines.
- · `global-subword-mode` Toggle Subword mode in many buffers.
- · `global-superword-mode` Toggle Superword mode in many buffers.
- · `global-tab-line-mode` Toggle Tab-Line mode in many buffers.
- · `global-text-scale-adjust` Change (a.k.a. "adjust") the font size of all faces by INCREMENT.
- · `global-unset-key` Remove global binding of KEY.
- · `global-visual-line-mode` Toggle Visual-Line mode in many buffers.
- · `global-visual-wrap-prefix-mode` Toggle Visual-Wrap-Prefix mode in many buffers.
- · `global-whitespace-mode` Toggle Whitespace mode in many buffers.
- · `global-whitespace-newline-mode` Toggle global newline visualization (Global Whitespace Newline mode).
- · `global-whitespace-toggle-options` Toggle global `whitespace-mode' options.
- · `global-window-tool-bar-mode` Toggle Window-Tool-Bar mode in many buffers.
- · `global-word-wrap-whitespace-mode` Toggle Word-Wrap-Whitespace mode in many buffers.
- · `global-xref-mouse-mode` Toggle Xref-Mouse mode in many buffers.

## glyphless

- · `glyphless-display-mode` Minor mode for displaying glyphless characters in the current buffer.

## gnus

- · `gnus` Read network news.
- · `gnus-agent-batch` Start Gnus, send queue and fetch session.
- · `gnus-agent-batch-fetch` Start Gnus and fetch session.
- · `gnus-agentize` Allow Gnus to be an offline newsreader.
- · `gnus-agent-regenerate` Regenerate all agent covered files.
- · `gnus-article-outlook-deuglify-article` Deuglify broken Outlook (Express) articles and redisplay.
- · `gnus-article-outlook-rearrange-citation` Repair broken citations.
- · `gnus-article-outlook-repair-attribution` Repair a broken attribution line.
- · `gnus-article-outlook-unwrap-lines` Unwrap lines that appear to be wrapped citation lines.
- · `gnus-batch-kill` Run batched scoring.
- · `gnus-batch-score` Run batched scoring.
- · `gnus-bookmark-bmenu-list` Display a list of existing Gnus bookmarks.
- · `gnus-bookmark-jump` Jump to a Gnus bookmark (BMK-NAME).
- · `gnus-bookmark-set` Set a bookmark for this article.
- · `gnus-button-reply` Like `message-reply'.
- · `gnus-cache-generate-active` Generate the cache active file.
- · `gnus-cache-generate-nov-databases` Generate NOV files recursively starting in DIR.
- · `gnus-child` Read news as a child.
- · `gnus-child-no-server` Read network news as a child, without connecting to the local server.
- · `gnus-child-unplugged` Read news as a child unplugged.
- · `gnus-delay-article` Delay this article by some time.
- · `gnus-delay-send-queue` Send all the delayed messages that are due now.
- · `gnus-draft-reminder` Reminder user if there are unsent drafts.
- · `gnus-face-from-file` Return a Face header based on an image FILE.
- · `gnus-fetch-group` Start Gnus if necessary and enter GROUP.
- · `gnus-fetch-group-other-frame` Pop up a frame and enter GROUP.
- · `gnus-group-split-setup` Set up the split for `nnmail-split-fancy'.
- · `gnus-group-split-update` Computes `nnmail-split-fancy' from group params and CATCH-ALL.
- · `gnus-insert-random-x-face-header` Insert a random X-Face header from `gnus-x-face-directory'.
- · `gnus-jog-cache` Go through all groups and put the articles into the cache.
- · `gnus-mailing-list-insinuate` Setup group parameters from List-Post header.
- · `gnus-mailing-list-mode` Minor mode for providing mailing-list commands.
- · `gnus-msg-mail` Start editing a mail message to be sent.
- · `gnus-no-server` Read network news.
- · `gnus-other-frame` Pop up a frame to read news.
- · `gnus-outlook-deuglify-article` Full deuglify of broken Outlook (Express) articles.
- · `gnus-plugged` Start Gnus plugged.
- · `gnus-random-face` Return randomly chosen Face from `gnus-face-directory'.
- · `gnus-random-x-face` Return X-Face header data chosen randomly from `gnus-x-face-directory'.
- · `gnus-read-ephemeral-emacs-bug-group` Browse Emacs bug reports with IDS in an ephemeral group.
- · `gnus-registry-initialize` Initialize the Gnus registry.
- · `gnus-score-mode` Mode for editing Gnus score files.
- · `gnus-sieve-article-add-rule` 
- · `gnus-sieve-generate` Generate the Sieve script in gnus-sieve-file, by replacing the region
- · `gnus-sieve-update` Update the Sieve script in gnus-sieve-file, by replacing the region
- · `gnus-slave` Read news as a child.
- · `gnus-slave-no-server` Read network news as a child, without connecting to the local server.
- · `gnus-slave-unplugged` Read news as a child unplugged.
- · `gnus-treat-from-gravatar` Display gravatar in the From header.
- · `gnus-treat-from-picon` Display picons in the From header.
- · `gnus-treat-mail-gravatar` Display gravatars in the Cc and To headers.
- · `gnus-treat-mail-picon` Display picons in the Cc and To headers.
- · `gnus-treat-newsgroups-picon` Display picons in the Newsgroups and Followup-To headers.
- · `gnus-unplugged` Start Gnus unplugged.
- · `gnus-update-format` Update the format specification near point.
- · `gnus-x-face-from-file` Insert an X-Face header based on an image FILE.

## go

- · `go-mod-ts-mode` Major mode for editing go.mod files, powered by tree-sitter.
- · `go-ts-mode` Major mode for editing Go, powered by tree-sitter.
- · `go-work-ts-mode` Major mode for editing go.work files, powered by tree-sitter.

## gomoku

- · `gomoku` Start a Gomoku game between you and Emacs.

## goto

- · `goto-address` Sets up goto-address functionality in the current buffer.
- · `goto-address-at-point` Compose a new message to the e-mail address or open URL at point.
- · `goto-address-mode` Minor mode to buttonize URLs and e-mail addresses in the current buffer.
- · `goto-address-prog-mode` Like `goto-address-mode', but only for comments and strings.
- ✓ `goto-char` Set point to POSITION, a number or marker.
- · `goto-history-element` Insert into the minibuffer the element of minibuffer history specified by NABS.
- ✓ `goto-line` Go to LINE, counting from line 1 at beginning of buffer.
- ✓ `goto-line-relative` Go to LINE, counting from line at (point-min).
- · `goto-next-locus` Visit next `next-error' message and corresponding source code.

## gpm

- · `gpm-mouse-mode` Toggle mouse support in GNU/Linux consoles (GPM Mouse mode).

## grep

- · `grep` Run Grep with user-specified COMMAND-ARGS.
- · `grep-find` Run grep via find, with user-specified args COMMAND-ARGS.
- · `grep-mode` Sets `grep-last-buffer' and `compilation-window-height'.

## gud

- · `gud-gdb` Run gdb passing it COMMAND-LINE as arguments.
- · `gud-tooltip-mode` Toggle the display of GUD tooltips.

## gui

- · `gui-set-selection` Make an X selection of type TYPE and value DATA.

## guiler

- · `guiler` Run guiler on program FILE in buffer `*gud-FILE*'.

## halfwidth

- · `halfwidth-region` Convert fullwidth characters in the region to their ASCII variants.
- · `halfwidth-word` Convert characters in word at point to fullwidth, moving over the word.

## handle

- · `handle-delete-frame` Handle delete-frame events from the X server.
- · `handle-focus-in` Handle a focus-in event.
- · `handle-focus-out` Handle a focus-out event.
- · `handle-move-frame` Handle a move-frame event.
- · `handle-save-session` Handle the save_yourself event from a session manager.
- · `handle-select-window` Handle select-window events.
- · `handle-switch-frame` Handle a switch-frame event EVENT.

## handwrite

- · `handwrite` Turn the buffer into a "handwritten" document.

## hanoi

- · `hanoi` Towers of Hanoi diversion.  Use NRINGS rings.
- · `hanoi-unix` Towers of Hanoi, 32-bit UNIX doomsday version.
- · `hanoi-unix-64` Like `hanoi-unix', but with a 64-bit clock.

## header

- · `header-line-indent-mode` Minor mode to help with alignment of header line when line numbers are shown.

## heex

- · `heex-ts-mode` Major mode for editing HEEx, powered by tree-sitter.

## help

- · `help` Help command.
- · `help-at-pt-cancel-timer` Cancel any timer set by `help-at-pt-set-timer'.
- · `help-at-pt-set-timer` Enable `help-at-pt-display-when-idle'.
- · `help-find-source` Switch to a buffer visiting the source of what is being described in *Help*.
- ✓ `help-for-help` Help command.
- · `help-for-help-internal` Help command.
- · `help-make-xrefs` Parse and hyperlink documentation cross-references in the given BUFFER.
- · `help-mode` Major mode for viewing help text and navigating references in it.
- · `help-quick` Display a quick-help buffer showing popular commands and their bindings.
- · `help-quick-toggle` Toggle display of a window showing popular commands and their bindings.
- ✓ `help-quit` Just exit from the Help command's command loop.
- ✓ `help-with-tutorial` Select the Emacs learn-by-doing tutorial.
- · `help-with-tutorial-spec-language` Use the Emacs tutorial, specifying which language you want.
- · `help-xref-interned` Display the full documentation of SYMBOL.

## Helper

- · `Helper-describe-bindings` Describe local key bindings of current mode.
- · `Helper-help` Provide help for current mode.

## hexl

- · `hexl-find-file` Edit file FILENAME as a binary file in hex dump format.
- · `hexl-mode` A mode for editing binary files in hex dump format.

## hexlify

- · `hexlify-buffer` Convert a binary buffer to hexl format.

## hi

- · `hi-lock-face-buffer` Set face of each match of REGEXP to FACE.
- · `hi-lock-face-phrase-buffer` Set face of each match of phrase REGEXP to FACE.
- · `hi-lock-face-symbol-at-point` Highlight each instance of the symbol at point.
- · `hi-lock-find-patterns` Add patterns from the current buffer to the list of hi-lock patterns.
- · `hi-lock-line-face-buffer` Highlight all lines that match REGEXP using FACE.
- · `hi-lock-mode` Toggle selective highlighting of patterns (Hi Lock mode).
- · `hi-lock-unface-buffer` Remove highlighting of each match to REGEXP set by hi-lock.
- · `hi-lock-write-interactive-patterns` Write interactively added patterns, if any, into buffer at point.

## hide

- · `hide-ifdef-mode` Toggle features to hide/show #ifdef blocks (Hide-Ifdef mode).

## highlight

- · `highlight-changes-mode` Toggle highlighting changes in this buffer (Highlight Changes mode).
- · `highlight-changes-next-change` Move to the beginning of the next change, if in Highlight Changes mode.
- · `highlight-changes-previous-change` Move to the beginning of the previous change, if in Highlight Changes mode.
- · `highlight-changes-remove-highlight` Remove the change face from the region between BEG and END.
- · `highlight-changes-rotate-faces` "Age" changes if in Highlight Changes mode and the changes are visible.
- · `highlight-changes-visible-mode` Toggle visibility of highlighting due to Highlight Changes mode.
- · `highlight-compare-buffers` Compare two buffers and highlight the differences.
- · `highlight-compare-with-file` Compare this buffer with a file, and highlight differences.
- · `highlight-confusing-reorderings` Highlight text in region that might be bidi-reordered in suspicious ways.
- ✓ `highlight-lines-matching-regexp` Highlight all lines that match REGEXP using FACE.
- ✓ `highlight-phrase` Set face of each match of phrase REGEXP to FACE.
- ✓ `highlight-regexp` Set face of each match of REGEXP to FACE.
- ✓ `highlight-symbol-at-point` Highlight each instance of the symbol at point.

## hippie

- · `hippie-expand` Try to expand text before point, using multiple methods.

## hl

- · `hl-line-mode` Toggle highlighting of the current line (Hl-Line mode).

## holiday

- · `holiday-list` Display holidays for years Y1 to Y2 (inclusive).

## holidays

- · `holidays` Display the holidays for last month, this month, and next month.

## horizontal

- · `horizontal-scroll-bar-mode` Toggle horizontal scroll bars on all frames (Horizontal Scroll Bar mode).

## how

- · `how-many` Print and return number of matches for REGEXP following point.

## hs

- · `hs-indentation-mode` Toggle indentation-based hiding/showing.
- · `hs-minor-mode` Minor mode to selectively hide/show code and comment blocks.

## html

- · `html-mode` Major mode based on SGML mode for editing HTML documents.
- · `html-ts-mode` Major mode for editing Html, powered by tree-sitter.

## htmlfontify

- · `htmlfontify-buffer` Create a new buffer, named for the current buffer + a .html extension,
- · `htmlfontify-copy-and-link-dir` Trawl SRCDIR and write fontified-and-hyperlinked output in DSTDIR.
- · `htmlfontify-load-rgb-file` Load an X11 style rgb.txt FILE.

## ibuffer

- · `ibuffer` Begin using Ibuffer to edit a list of buffers.
- · `ibuffer-jump` Call Ibuffer and set point at the line listing the current buffer.
- · `ibuffer-list-buffers` Display a list of buffers, in another window.
- · `ibuffer-other-window` Like `ibuffer', but displayed in another window by default.

## icalendar

- · `icalendar-export-file` Export diary file to iCalendar format.
- · `icalendar-export-region` Export region in diary file to iCalendar format.
- · `icalendar-import-buffer` Extract iCalendar events from current buffer.
- · `icalendar-import-file` Import an iCalendar file and append to a diary file.
- · `icalendar-mode` Major mode for viewing and editing iCalendar (RFC5545) data.

## icomplete

- · `icomplete-mode` Toggle incremental minibuffer completion (Icomplete mode).
- · `icomplete-vertical-mode` Toggle vertical candidate display in `icomplete-mode' or `fido-mode'.

## icon

- · `icon-mode` Major mode for editing Icon code.

## iconify

- · `iconify-frame` Make the frame FRAME into an icon.
- · `iconify-or-deiconify-frame` Iconify the selected frame, or deiconify if it's currently an icon.

## idl

- · `idl-mode` Major mode for editing CORBA's IDL, PSDL and CIDL code.

## ido

- · `ido-dired` Call `dired' the Ido way.
- · `ido-dired-other-frame` "Edit" a directory.  Like `ido-dired' but make a new frame.
- · `ido-dired-other-window` "Edit" a directory.  Like `ido-dired' but select in another window.
- · `ido-display-buffer` Display a buffer in another window but don't select it.
- · `ido-display-buffer-other-frame` Display a buffer preferably in another frame.
- · `ido-display-file` Display a file in another window but don't select it.
- · `ido-find-alternate-file` Find another file, select its buffer, kill previous buffer.
- · `ido-find-alternate-file-other-window` Find file as a replacement for the file in the next window.
- · `ido-find-file` Edit file with name obtained via minibuffer.
- · `ido-find-file-in-dir` Switch to another file starting from DIR.
- · `ido-find-file-other-frame` Switch to another file and show it in another frame.
- · `ido-find-file-other-window` Switch to another file and show it in another window.
- · `ido-find-file-read-only` Edit file read-only with name obtained via minibuffer.
- · `ido-find-file-read-only-other-frame` Edit file read-only in other frame with name obtained via minibuffer.
- · `ido-find-file-read-only-other-window` Edit file read-only in other window with name obtained via minibuffer.
- · `ido-insert-buffer` Insert contents of a buffer in current buffer after point.
- · `ido-insert-file` Insert contents of file in current buffer.
- · `ido-kill-buffer` Kill a buffer.
- · `ido-mode` Toggle Ido mode on or off.
- · `ido-switch-buffer` Switch to another buffer.
- · `ido-switch-buffer-other-frame` Switch to another buffer and show it in another frame.
- · `ido-switch-buffer-other-window` Switch to another buffer and show it in another window.
- · `ido-write-file` Write current buffer to a file.

## ielm

- · `ielm` Interactively evaluate Emacs Lisp expressions.

## ifconfig

- · `ifconfig` Run `ifconfig-program' and display diagnostic output.

## ignore

- · `ignore` Ignore ARGUMENTS, do nothing, and return nil.
- · `ignore-preserving-kill-region` Like `ignore', but don't overwrite `last-event' if it's `kill-region'.

## iimage

- · `iimage-mode` Toggle Iimage mode on or off.

## image

- · `image-crop` Crop the image under point.
- · `image-cut` Cut a rectangle from the image under point, filling it with COLOR.
- · `image-decrease-size` Decrease the image size at POSITION by a factor specified by N.
- · `image-dired` Make a thumbnail buffer for all images in DIRNAME and display it.
- · `image-dired-delete-tag` Remove tag for selected file(s).
- · `image-dired-dired-comment-files` Add comment to current or marked files in Dired.
- · `image-dired-dired-display-external` Display file at point using an external viewer.
- · `image-dired-dired-display-image` Display current image file.
- · `image-dired-dired-edit-comment-and-tags` Edit comment and tags of current or marked image files.
- · `image-dired-dired-toggle-marked-thumbs` Toggle thumbnails in front of marked file names in the Dired buffer.
- · `image-dired-dired-with-window-configuration` Open directory DIR and create a default window configuration.
- · `image-dired-display-thumb` Shorthand for `image-dired-display-thumbs' with prefix argument.
- · `image-dired-display-thumbs` Display thumbnails of all marked files, in `image-dired-thumbnail-buffer'.
- · `image-dired-display-thumbs-append` Append thumbnails to `image-dired-thumbnail-buffer'.
- · `image-dired-jump-thumbnail-buffer` Jump to thumbnail buffer.
- · `image-dired-mark-tagged-files` Mark files whose tag matches REGEXP.
- · `image-dired-minor-mode` Setup easy-to-use keybindings for Image-Dired in Dired mode.
- · `image-dired-setup-dired-keybindings` Setup easy-to-use keybindings for Image-Dired in Dired mode.
- · `image-dired-show-all-from-dir` Make a thumbnail buffer for all images in DIRNAME and display it.
- · `image-dired-tag-files` Tag file(s) which are marked in a Dired buffer.
- · `image-flip-horizontally` Horizontally flip the image under point.
- · `image-flip-vertically` Vertically flip the image under point.
- · `image-increase-size` Increase the image size at POSITION by a factor specified by N.
- · `image-minor-mode` Toggle Image minor mode in this buffer.
- · `image-mode` Major mode for image files.
- · `image-mouse-decrease-size` Decrease the image size using the mouse-gesture EVENT.
- · `image-mouse-increase-size` Increase the image size using the mouse-gesture EVENT.
- · `image-rotate` Rotate the image under point by ANGLE degrees clockwise.
- · `image-save` Save the image under point.

## imenu

- ✓ `imenu` Jump to a place in the buffer chosen using a buffer menu or mouse menu.
- · `imenu-add-menubar-index` Add an Imenu "Index" entry on the menu bar for the current buffer.
- · `imenu-add-to-menubar` Add an `imenu' entry to the menu bar for the current buffer.

## increase

- · `increase-left-margin` Increase or decrease the `left-margin' of the region.
- · `increase-right-margin` Increase the right-margin of the region.

## increment

- ✓ `increment-register` Augment contents of REGISTER using PREFIX.

## indent

- · `indent-according-to-mode` Indent line in proper way for current major mode.
- · `indent-code-rigidly` Indent all lines of code, starting in the region, sideways by ARG columns.
- · `indent-for-comment` Indent this line's comment to `comment-column', or insert an empty comment.
- ✓ `indent-for-tab-command` Indent the current line or region, or insert a tab, as appropriate.
- · `indent-new-comment-line` Break line at point and indent, continuing comment if within one.
- · `indent-pp-sexp` Indent each line of the list starting just after point, or prettyprint it.
- ✓ `indent-region` Indent each nonblank line in the region.
- · `indent-relative` Space out to under next indent point in previous nonblank line.
- · `indent-relative-first-indent-point` Indent the current line like the previous nonblank line.
- · `indent-relative-maybe` Indent the current line like the previous nonblank line.
- ✓ `indent-rigidly` Indent all lines starting in the region.
- ✓ `indent-rigidly-left` Indent all lines between BEG and END leftward by one space.
- · `indent-rigidly-left-to-tab-stop` Indent all lines between BEG and END leftward to a tab stop.
- ✓ `indent-rigidly-right` Indent all lines between BEG and END rightward by one space.
- · `indent-rigidly-right-to-tab-stop` Indent all lines between BEG and END rightward to a tab stop.
- · `indent-sexp` Indent each line of the list starting just after point.
- · `indent-tabs-mode` Toggle whether indentation can insert TAB characters.
- · `indent-to` Indent from point with tabs and spaces until COLUMN is reached.
- · `indent-to-column` Indent from point with tabs and spaces until COLUMN is reached.

## indented

- · `indented-text-mode` Major mode for editing text written for humans to read.

## indian

- · `indian-2-column-to-ucs-region` Convert old Emacs Devanagari characters to UCS.
- · `indian-compose-region` Compose the region according to `composition-function-table'.

## inferior

- · `inferior-lisp` Run an inferior Lisp process, input and output via buffer `*inferior-lisp*'.
- · `inferior-octave` Run an inferior Octave process, I/O via `inferior-octave-buffer'.
- · `inferior-tcl` Run inferior Tcl process.

## info

- ✓ `info` Enter Info, the documentation browser.
- · `info-apropos` Search indices of all known Info files on your system for STRING.
- · `info-complete-file` Perform completion on file preceding point.
- · `info-complete-symbol` Perform completion on symbol preceding point.
- ✓ `info-display-manual` Display an Info buffer displaying MANUAL.
- · `info-emacs-bug` Display the "Reporting Bugs" section of the Emacs manual in Info mode.
- ✓ `info-emacs-manual` Display the Emacs manual in Info mode.
- · `info-finder` Display descriptions of the keywords in the Finder virtual manual.
- · `info-lookup-file` Look up and display documentation of FILE in the relevant Info manual.
- · `info-lookup-reset` Throw away all cached data.
- ✓ `info-lookup-symbol` Look up and display documentation of SYMBOL in the relevant Info manual.
- ✓ `info-other-window` Like `info' but show the Info buffer in another window.
- · `info-xref-check` Check external references in FILENAME, an info document.
- · `info-xref-check-all` Check external references in all info documents in the info path.
- · `info-xref-check-all-custom` Check info references in all customize groups and variables.
- · `info-xref-docstrings` Check docstring info node references in source files.

## Info

- · `Info-directory` Go to the Info directory node.
- ✓ `Info-goto-emacs-command-node` Go to the Info node in the Emacs manual for command COMMAND.
- ✓ `Info-goto-emacs-key-command-node` Go to the node in the Emacs manual which describes the command bound to KEY.
- · `Info-index` Look up a string TOPIC in the index for this manual and go to that entry.
- · `Info-mode` Info mode provides commands for browsing through the Info documentation tree.
- · `Info-on-current-buffer` Use Info mode to browse the current Info buffer.
- · `Info-speedbar-browser` Initialize speedbar to display an Info node browser.
- · `Info-split` Split an info file into an indirect file plus bounded-size subfiles.
- · `Info-tagify` Create or update Info file tag table in current buffer or in a region.
- · `Info-validate` Check current buffer for validity as an Info file.

## inhibit

- · `inhibit-remote-files` Deactivate remote file names.

## insert

- · `insert-abbrevs` Insert the description of all defined abbrevs after point.
- ✓ `insert-buffer` Insert after point the contents of BUFFER.
- · `insert-char` Insert COUNT copies of CHARACTER.
- ✓ `insert-file` Insert contents of file FILENAME into buffer after point.
- · `insert-file-literally` Insert contents of file FILENAME into buffer after point with no conversion.
- · `insert-kbd-macro` Insert in buffer the definition of kbd macro MACRONAME, as Lisp code.
- · `insert-pair` Enclose following ARG sexps in a pair of OPEN and CLOSE characters.
- ✓ `insert-parentheses` Enclose following ARG sexps in parentheses.
- ✓ `insert-register` Insert contents of REGISTER at point.

## inverse

- · `inverse-add-global-abbrev` Define the word before point as a global (mode-independent) abbreviation.
- · `inverse-add-mode-abbrev` Define the word before point as a mode-specific abbreviation.

## invert

- · `invert-face` Swap the foreground and background colors of FACE.

## irc

- · `irc` Connect to all servers in `rcirc-server-alist'.

## isearch

- ✓ `isearch-abort` Abort incremental search mode if searching is successful, signaling quit.
- ✓ `isearch-backward` Do incremental search backward.
- · `isearch-backward-regexp` Do incremental search backward for regular expression.
- · `isearch-beginning-of-buffer` Go to the first occurrence of the current search string.
- · `isearch-cancel` Terminate the search and go back to the starting point.
- · `isearch-char-by-name` Read a character by its Unicode name and add it to the search string.
- · `isearch-complete` Complete the search string from the strings on the search ring.
- · `isearch-complete-edit` Same as `isearch-complete' except in the minibuffer.
- · `isearch-del-char` Delete character from end of search string and search again.
- ✓ `isearch-delete-char` Undo last input item during a search.
- · `isearch-describe-bindings` Show a list of all keys defined in Isearch mode, and their definitions.
- · `isearch-describe-key` Display documentation of the function invoked by isearch key.
- · `isearch-describe-mode` Display documentation of Isearch mode.
- · `isearch-edit-string` Edit the search string in the minibuffer.
- · `isearch-emoji-by-name` Read an Emoji name and add it to the search string COUNT times.
- · `isearch-end-of-buffer` Go to the last occurrence of the current search string.
- ✓ `isearch-exit` Exit search normally.
- · `isearch-fold-quotes-mode` Minor mode to aid searching for \=` characters in help modes.
- ✓ `isearch-forward` Do incremental search forward.
- · `isearch-forward-exit-minibuffer` Resume isearching forward from the minibuffer that edits the search string.
- · `isearch-forward-regexp` Do incremental search forward for regular expression.
- · `isearch-forward-symbol` Do incremental search forward for a symbol.
- ✓ `isearch-forward-symbol-at-point` Do incremental search forward for a symbol found near point.
- · `isearch-forward-thing-at-point` Do incremental search forward for the "thing" found near point.
- · `isearch-forward-word` Do incremental search forward for a sequence of words.
- · `isearch-help-for-help` Display Isearch help menu.
- · `isearch-help-for-help-internal` Help command.
- · `isearch-highlight-lines-matching-regexp` Exit Isearch mode and call `highlight-lines-matching-regexp'.
- · `isearch-highlight-regexp` Exit Isearch mode and call `highlight-regexp'.
- · `isearch-menu-bar-map` Menu for `isearch-mode'.
- · `isearch-mode-help` Display documentation of Isearch mode.
- · `isearch-mouse-2` Handle mouse-2 in Isearch mode.
- · `isearch-nonincremental-exit-minibuffer` 
- · `isearch-occur` Run `occur' using the last search string as the regexp.
- · `isearch-printing-char` Add this ordinary printing CHAR to the search string and search.
- · `isearch-query-replace` Start `query-replace' with string to replace from last search string.
- · `isearch-query-replace-regexp` Start `query-replace-regexp' with string to replace from last search string.
- · `isearch-quote-char` Quote special characters for incremental search.
- ✓ `isearch-repeat-backward` Repeat incremental search backwards.
- ✓ `isearch-repeat-forward` Repeat incremental search forwards.
- · `isearch-reverse-exit-minibuffer` Resume isearching backward from the minibuffer that edits the search string.
- · `isearch-ring-advance` Advance to the next search string in the ring.
- · `isearch-ring-retreat` Retreat to the previous search string in the ring.
- · `isearch-tmm-menubar` Run `tmm-menubar' while `isearch-mode' is enabled.
- · `isearch-toggle-case-fold` Toggle case-fold searching on or off.
- · `isearch-toggle-char-fold` Toggle char-fold searching on or off.
- · `isearch-toggle-input-method` Toggle input method in interactive search.
- · `isearch-toggle-invisible` Toggle invisible searching on or off.
- · `isearch-toggle-lax-whitespace` Toggle lax-whitespace searching on or off.
- · `isearch-toggle-regexp` Toggle regexp searching on or off.
- · `isearch-toggle-specified-input-method` Select an input method and turn it on in interactive search.
- · `isearch-toggle-symbol` Toggle symbol searching on or off.
- · `isearch-toggle-word` Toggle word searching on or off.
- · `isearch-transient-input-method` Activate transient input method in interactive search.
- · `isearch-xterm-paste` Pull terminal paste into search string.
- · `isearch-yank-char` Pull next character from buffer into search string.
- · `isearch-yank-char-in-minibuffer` Pull next character from buffer into end of search string in minibuffer.
- · `isearch-yank-from-kill-ring` Read a string from the `kill-ring' and append it to the search string.
- · `isearch-yank-kill` Pull string from kill ring into search string.
- · `isearch-yank-line` Pull rest of line from buffer into search string.
- · `isearch-yank-pop` Replace just-yanked search string with previously killed string.
- · `isearch-yank-pop-only` Replace just-yanked search string with previously killed string.
- · `isearch-yank-symbol-or-char` Pull next character or symbol from buffer into search string.
- · `isearch-yank-until-char` Pull everything until next instance of CHAR from buffer into search string.
- · `isearch-yank-word` Pull next word from buffer into search string.
- · `isearch-yank-word-or-char` Pull next character or word from buffer into search string.
- · `isearch-yank-x-selection` Pull current PRIMARY X selection into the search string.

## isearchb

- · `isearchb-activate` Active isearchb mode for subsequent alphanumeric keystrokes.

## iso

- · `iso-cvt-define-menu` Add submenus to the File menu, to convert to and from various formats.
- · `iso-cvt-read-only` Warn that format is read-only.
- · `iso-cvt-write-only` Warn that format is write-only.
- · `iso-german` Translate net conventions for German to ISO 8859-1.
- · `iso-gtex2iso` Translate German TeX sequences to ISO 8859-1 characters.
- · `iso-iso2duden` Translate ISO 8859-1 characters to Duden sequences.
- · `iso-iso2gtex` Translate ISO 8859-1 characters to German TeX sequences.
- · `iso-iso2sgml` Translate ISO 8859-1 characters in the region to SGML entities.
- · `iso-iso2tex` Translate ISO 8859-1 characters to TeX sequences.
- · `iso-sgml2iso` Translate SGML entities in the region to ISO 8859-1 characters.
- · `iso-spanish` Translate net conventions for Spanish to ISO 8859-1.
- · `iso-tex2iso` Translate TeX sequences to ISO 8859-1 characters.
- · `iso-transl-set-language` Set shorter key bindings for some characters relevant for LANG.

## ispell

- · `ispell` Interactively check a region or buffer for spelling errors.
- · `ispell-buffer` Check the current buffer for spelling errors interactively.
- · `ispell-buffer-with-debug` `ispell-buffer' with some output sent to `ispell-debug-buffer'.
- · `ispell-change-dictionary` Change to dictionary DICT for Ispell.
- · `ispell-comment-or-string-at-point` Check the comment or string containing point for spelling errors.
- · `ispell-comments-and-strings` Check comments and strings in the current buffer for spelling errors.
- · `ispell-complete-word` Try to complete the word before or at point.
- · `ispell-complete-word-interior-frag` Completes word matching character sequence inside a word.
- · `ispell-continue` Continue a halted spelling session beginning with the current word.
- · `ispell-kill-ispell` Kill current Ispell process (so that you may start a fresh one).
- · `ispell-message` Check the spelling of a mail message or news post.
- · `ispell-minor-mode` Toggle last-word spell checking (Ispell minor mode).
- · `ispell-pdict-save` Check to see if the personal dictionary has been modified.
- · `ispell-region` Interactively check region between REG-START and REG-END for spelling errors.
- ✓ `ispell-word` Check spelling of word under or before the cursor.

## iswitchb

- · `iswitchb-mode` Toggle Iswitchb mode.

## iwconfig

- · `iwconfig` Run `iwconfig-program' and display diagnostic output.

## japanese

- · `japanese-hankaku-region` Convert Japanese `zenkaku' chars in the region to `hankaku' chars.
- · `japanese-hiragana-region` Convert Japanese `katakana' chars in the region to `hiragana' chars.
- · `japanese-katakana-region` Convert Japanese `hiragana' chars in the region to `katakana' chars.
- · `japanese-zenkaku-region` Convert hankaku' chars in the region to Japanese `zenkaku' chars.

## java

- · `java-mode` Major mode for editing Java code.
- · `java-ts-mode` Major mode for editing Java, powered by tree-sitter.

## javascript

- · `javascript-mode` Major mode for editing JavaScript.

## jdb

- · `jdb` Run jdb with command line COMMAND-LINE in a buffer.

## jit

- · `jit-lock-debug-mode` Minor mode to help debug code run from jit-lock.

## join

- ✓ `join-line` Join this line to previous and fix up whitespace at join.

## js

- · `js-base-mode` Generic major mode for editing JavaScript.
- · `js-json-mode` Major mode derived from `prog-mode' by `define-derived-mode'.
- · `js-jsx-mode` Major mode for editing JavaScript+JSX.
- · `js-mode` Major mode for editing JavaScript.
- · `js-ts-mode` Major mode for editing JavaScript.

## json

- · `json-ts-mode` Major mode for editing JSON, powered by tree-sitter.

## jump

- ✓ `jump-to-register` Go to location stored in REGISTER, or restore configuration stored there.

## just

- ✓ `just-one-space` Delete all spaces and tabs around point, leaving one space (or N spaces).

## justify

- · `justify-current-line` Do some kind of justification on this line.

## kbd

- · `kbd-macro-query` Query user during kbd macro execution.

## keep

- · `keep-lines` Delete all lines except those containing matches for REGEXP.

## key

- · `key-translate` Translate character FROM to TO on the current terminal.
- · `key-translate-remove` Remove translation of FROM from `keyboard-translate-table'.

## keyboard

- ✓ `keyboard-escape-quit` Exit the current "mode" (in a generalized sense of the word).
- ✓ `keyboard-quit` Signal a `quit' condition.

## keymap

- · `keymap-global-lookup` Return the binding for command KEYS in current global keymap only.
- · `keymap-global-set` Give KEY a global binding as COMMAND.
- · `keymap-global-unset` Remove global binding of KEY (if any).
- · `keymap-local-set` Give KEY a local binding as COMMAND.
- · `keymap-local-unset` Remove local binding of KEY (if any).

## kill

- · `kill-all-abbrevs` Undefine all defined abbrevs.
- · `kill-backward-up-list` Kill the form containing the current sexp, leaving the sexp itself.
- ✓ `kill-buffer` Kill the buffer specified by BUFFER-OR-NAME.
- ✓ `kill-buffer-and-window` Kill the current buffer and delete the selected window.
- · `kill-comment` Kill the first comment on this line, if any.
- · `kill-current-buffer` Kill the current buffer.
- · `kill-emacs` Exit the Emacs job and kill it.
- ✓ `kill-line` Kill the rest of the current line; if no nonblanks there, kill thru newline.
- · `kill-local-variable` Make VARIABLE no longer have a separate value in the current buffer.
- · `kill-matching-buffers` Kill buffers whose names match the regular expression REGEXP.
- · `kill-matching-buffers-no-ask` Kill buffers whose names match the regular expression REGEXP.
- · `kill-matching-lines` Kill lines containing matches for REGEXP.
- · `kill-paragraph` Kill forward to end of paragraph.
- · `kill-process` Kill process PROCESS.  May be process or name of one.
- ✓ `kill-rectangle` Delete the region-rectangle and save it as the last killed one.
- ✓ `kill-region` Kill ("cut") text between point and mark.
- · `kill-ring-deindent-mode` Toggle removal of indentation from text saved to the kill ring.
- ✓ `kill-ring-save` Save the region as if killed, but don't kill it.
- ✓ `kill-sentence` Kill from point to end of sentence.
- ✓ `kill-sexp` Kill the sexp (balanced expression) following point.
- · `kill-some-buffers` Kill some buffers.  Asks the user whether to kill each one of them.
- · `kill-this-buffer` Kill the current buffer.
- · `kill-visual-line` Kill the rest of the visual line.
- ✓ `kill-whole-line` Kill current line.
- ✓ `kill-word` Kill characters forward until encountering the end of a word.

## kkc

- · `kkc-region` Convert Kana string in the current region to Kanji-Kana mixed string.

## kmacro

- · `kmacro-call-macro` Call the keyboard MACRO that you defined with \[kmacro-start-macro].
- ✓ `kmacro-end-and-call-macro` Call last keyboard macro, ending it first if currently being defined.
- · `kmacro-end-call-mouse` Call last keyboard macro, ending it first if currently being defined.
- ✓ `kmacro-end-macro` Finish defining a keyboard macro.
- ✓ `kmacro-end-or-call-macro` End kbd macro if currently being defined; else call last kbd macro.
- · `kmacro-keymap` Keymap for keyboard macro commands.
- · `kmacro-menu` List the keyboard macros.
- · `kmacro-name-last-macro` Assign a name to the last keyboard macro defined.
- ✓ `kmacro-start-macro` Record subsequent keyboard input, defining a keyboard macro.
- ✓ `kmacro-start-macro-or-insert-counter` Record subsequent keyboard input, defining a keyboard macro.

## lao

- · `lao-compose-region` 

## last

- · `last-completion` Move to the last item in the completions buffer.

## latex

- · `latex-mode` Major mode for editing files of input for LaTeX.

## LaTeX

- · `LaTeX-mode` Major mode for editing files of input for LaTeX.

## latin1

- · `latin1-display-ucs-per-lynx` Set up Latin-1/ASCII display for Unicode characters.

## lazy

- · `lazy-highlight-cleanup` Stop lazy highlighting and remove extra highlighting from current buffer.

## ld

- · `ld-script-mode` A major mode to edit GNU ld script files.

## left

- ✓ `left-char` Move point N characters to the left (to the right if N is negative).
- ✓ `left-word` Move point N words to the left (to the right if N is negative).

## less

- · `less-css-mode` Major mode for editing Less files (https://lesscss.org/).

## lgrep

- · `lgrep` Run grep, searching for REGEXP in FILES in directory DIR.

## life

- · `life` Run Conway's Life simulation.

## line

- · `line-number-mode` Toggle line number display in the mode line (Line Number mode).

## lisp

- · `lisp-complete-symbol` Perform completion on Lisp symbol preceding point.
- · `lisp-data-mode` Major mode for buffers holding data written in Lisp syntax.
- · `lisp-eval-defun` 
- · `lisp-fill-paragraph` Like \[fill-paragraph], but handle Emacs Lisp comments and docstrings.
- · `lisp-indent-line` Indent current line as Lisp code.
- · `lisp-interaction-mode` Major mode for typing and evaluating Lisp forms.
- · `lisp-interaction-mode-menu` Menu for Lisp Interaction mode.
- · `lisp-mode` Major mode for editing programs in Common Lisp and other similar Lisps.
- · `lisp-mode-menu` Menu for ordinary Lisp mode.

## list

- · `list-abbrevs` Display a list of the defined abbrevs.
- · `list-bookmarks` Display a list of existing bookmarks.
- ✓ `list-buffers` Display a list of existing buffers.
- · `list-character-sets` Display a list of all character sets.
- · `list-charset-chars` Display a list of characters in character set CHARSET.
- · `list-coding-systems` Display a list of all coding systems.
- · `list-colors-display` Display names of defined colors, and show what they look like.
- · `list-command-history` List history of commands that used the minibuffer.
- ✓ `list-directory` Display a list of files in or matching DIRNAME, a la `ls'.
- · `list-dynamic-libraries` Display a list of all dynamic libraries known to Emacs.
- · `list-faces-display` List all faces, using the same sample text in each.
- · `list-fontsets` Display a list of all fontsets.
- · `list-holidays` Display holidays for years Y1 to Y2 (inclusive).
- · `list-input-methods` Display information about all input methods.
- · `list-keyboard-macros` List the keyboard macros.
- · `list-load-path-shadows` Display a list of Emacs Lisp files that shadow other files.
- · `list-matching-lines` Show all lines in the current buffer containing a match for REGEXP.
- · `list-multisession-values` List all values in the "multisession" database.
- · `list-packages` Display a list of packages.
- · `list-processes` Display a list of all processes that are Emacs sub-processes.
- · `list-registers` Display the list of nonempty registers with brief descriptions of contents.
- · `list-tags` Display list of tags in file FILE.
- · `list-threads` Display a list of threads.
- · `list-timers` List all timers in a buffer.

## lldb

- · `lldb` Run LLDB passing it COMMAND-LINE as arguments.

## load

- · `load-file` Load the Lisp file named FILE.
- · `load-library` Load the Emacs Lisp library named LIBRARY.
- · `load-theme` Load Custom theme named THEME from its file and possibly enable it.

## local

- · `local-set-key` Give KEY a local binding as COMMAND.
- · `local-unset-key` Remove local binding of KEY.

## locate

- · `locate` Run the program `locate', putting results in `*Locate*' buffer.
- · `locate-library` Show the precise file name of Emacs library LIBRARY.
- · `locate-with-filter` Run the executable program `locate' with a filter.

## lock

- · `lock-file-mode` Toggle file locking in the current buffer (Lock File mode).

## log

- · `log-view-mode` Major mode for browsing CVS log output.

## lossage

- · `lossage-size` Return or set the maximum number of keystrokes to save.

## lost

- · `lost-selection-mode` Toggle `lost-selection-mode'.

## lower

- · `lower-frame` Send FRAME to the back, so it is occluded by any frames that overlap it.

## lpr

- · `lpr-buffer` Print buffer contents without pagination or page headers.
- · `lpr-customize` Customization of the `lpr' group.
- · `lpr-region` Print region contents without pagination or page headers.

## lua

- · `lua-mode` Major mode for editing Lua code.
- · `lua-start-process` Start a Lua process named NAME, running PROGRAM.
- · `lua-ts-inferior-lua` Run a Lua interpreter in an inferior process.
- · `lua-ts-mode` Major mode for editing Lua files, powered by tree-sitter.

## lunar

- · `lunar-phases` Display the quarters of the moon for last month, this month, and next month.

## m2

- · `m2-mode` This is a mode intended to support program development in Modula-2.

## m4

- · `m4-mode` A major mode to edit m4 macro files.

## mail

- · `mail` Edit a message to be sent.  Prefix arg means resume editing (don't erase).
- · `mail-abbrevs-mode` Toggle abbrev expansion of mail aliases (Mail Abbrevs mode).
- · `mail-mode` Major mode for editing mail to be sent.
- · `mail-other-frame` Like `mail' command, but display mail buffer in another frame.
- · `mail-other-window` Like `mail' command, but display mail buffer in another window.
- · `mail-quote-printable-region` Convert the region to the "quoted printable" Q encoding.
- · `mail-send-and-exit` Send message like `mail-send', then, if no errors, exit from mail buffer.
- · `mail-unquote-printable-region` Undo the "quoted printable" encoding in buffer from BEG to END.

## mairix

- · `mairix-edit-saved-searches` Edit current mairix searches.
- · `mairix-edit-saved-searches-customize` Edit the list of saved searches in a customization buffer.
- · `mairix-search` Call Mairix with SEARCH.
- · `mairix-search-from-this-article` Search messages from sender of the current article.
- · `mairix-search-thread-this-article` Search thread for the current article.
- · `mairix-update-database` Call mairix for updating the database for SERVERS.
- · `mairix-use-saved-search` Use a saved search for querying Mairix.
- · `mairix-widget-search` Create mairix query interactively using graphical widgets.
- · `mairix-widget-search-based-on-article` Create mairix query based on current article using widgets.

## make

- · `make-directory` Create the directory DIR and optionally any nonexistent parent dirs.
- · `make-directory-autoloads` 
- · `make-empty-face` Define a new, empty face with name FACE.
- · `make-empty-file` Create an empty file FILENAME.
- · `make-face` Define a new face with name FACE, a symbol.
- · `make-face-bold` Make the font of FACE be bold, if possible.
- · `make-face-bold-italic` Make the font of FACE be bold and italic, if possible.
- · `make-face-italic` Make the font of FACE be italic, if possible.
- · `make-face-unbold` Make the font of FACE be non-bold, if possible.
- · `make-face-unitalic` Make the font of FACE be non-italic, if possible.
- · `make-frame` Return a newly created frame displaying the current buffer.
- · `make-frame-command` Make a new frame, on the same terminal as the selected frame.
- · `make-frame-invisible` Make the frame FRAME invisible.
- · `make-frame-on-current-monitor` Make a frame on the currently selected monitor.
- · `make-frame-on-display` Make a frame on display DISPLAY.
- · `make-frame-on-monitor` Make a frame on monitor MONITOR.
- · `make-frame-visible` Make the frame FRAME visible (assuming it is an X window).
- · `make-indirect-buffer` Create and return an indirect buffer for buffer BASE-BUFFER, named NAME.
- · `make-local-variable` Make VARIABLE have a separate value in the current buffer.
- · `make-symbolic-link` Make a symbolic link to TARGET, named LINKNAME.
- · `make-variable-buffer-local` Make VARIABLE become buffer-local whenever it is set.

## makefile

- · `makefile-automake-mode` An adapted `makefile-mode' that knows about automake.
- · `makefile-bsdmake-mode` An adapted `makefile-mode' that knows about BSD make.
- · `makefile-gmake-mode` An adapted `makefile-mode' that knows about gmake.
- · `makefile-imake-mode` An adapted `makefile-mode' that knows about imake.
- · `makefile-makepp-mode` An adapted `makefile-mode' that knows about makepp.
- · `makefile-mode` Major mode for editing standard Makefiles.

## malloc

- · `malloc-info` Report malloc information to stderr.
- · `malloc-trim` Release free heap memory to the OS.

## man

- · `man` Get a Un*x manual page and put it in a buffer.
- · `man-follow` Get a Un*x manual page of the item under point and put it in a buffer.

## manual

- · `manual-entry` Get a Un*x manual page and put it in a buffer.

## map

- · `map-query-replace-regexp` Replace some matches for REGEXP with various strings, in rotation.

## mark

- · `mark-beginning-of-buffer` Set mark at the beginning of the buffer.
- ✓ `mark-defun` Put mark at end of this defun, point at beginning.
- · `mark-end-of-buffer` Set mark at the end of the buffer.
- · `mark-end-of-sentence` Put mark at end of sentence.
- ✓ `mark-page` Put mark at end of page, point at beginning.
- ✓ `mark-paragraph` Put point at beginning of this paragraph, mark at end.
- ✓ `mark-sexp` Set mark ARG sexps from point or move mark one sexp.
- ✓ `mark-whole-buffer` Put point at beginning and mark at end of buffer.
- ✓ `mark-word` Set mark ARG words from point or move mark one word.

## master

- · `master-mode` Toggle Master mode.

## maximize

- · `maximize-window` Maximize WINDOW.

## memory

- · `memory-report` Generate a report of how Emacs is using memory.

## menu

- · `menu-bar-bottom-and-right-window-divider` Display dividers on the bottom and right of each window.
- · `menu-bar-bottom-window-divider` Display dividers only at the bottom of each window.
- · `menu-bar--display-line-numbers-mode-absolute` Turn on absolute line number mode.
- · `menu-bar--display-line-numbers-mode-none` Disable line numbers.
- · `menu-bar--display-line-numbers-mode-relative` Turn on relative line number mode.
- · `menu-bar--display-line-numbers-mode-visual` Turn on visual line number mode.
- · `menu-bar-enable-clipboard` Make CUT, PASTE and COPY (keys and menu bar items) use the clipboard.
- · `menu-bar-left-scroll-bar` Display scroll bars on the left of each window.
- · `menu-bar-mode` Toggle display of a menu bar on each frame (Menu Bar mode).
- · `menu-bar-no-scroll-bar` Turn off scroll bars.
- · `menu-bar-no-window-divider` Do not display window dividers.
- · `menu-bar-open` Start key navigation of the menu bar in FRAME.
- · `menu-bar-open-mouse` Open the menu bar for the menu item clicked on by the mouse.
- · `menu-bar-options-save` Save current values of Options menu items using Custom.
- · `menu-bar-read-lispintro` Display the Introduction to Emacs Lisp Programming in Info mode.
- · `menu-bar-read-lispref` Display the Emacs Lisp Reference manual in Info mode.
- · `menu-bar-read-mail` Read mail using `read-mail-command'.
- · `menu-bar-right-scroll-bar` Display scroll bars on the right of each window.
- · `menu-bar-right-window-divider` Display dividers only on the right of each window.
- · `menu-bar-select-buffer` 
- · `menu-bar-select-yank` Insert the stretch of previously-killed text selected from menu.
- · `menu-bar-showhide-fringe-ind-box` Display top and bottom indicators in opposite fringes.
- · `menu-bar-showhide-fringe-ind-customize` Show customization buffer for `indicate-buffer-boundaries'.
- · `menu-bar-showhide-fringe-ind-left` Display buffer boundaries and arrows in the left fringe.
- · `menu-bar-showhide-fringe-ind-mixed` Display top and bottom indicators in opposite fringes, arrows in right.
- · `menu-bar-showhide-fringe-ind-none` Do not display any buffer boundary indicators.
- · `menu-bar-showhide-fringe-ind-right` Display buffer boundaries and arrows in the right fringe.
- · `menu-bar-showhide-fringe-menu-customize` Show customization buffer for `fringe-mode'.
- · `menu-bar-showhide-fringe-menu-customize-disable` Do not display window fringes.
- · `menu-bar-showhide-fringe-menu-customize-left` Display fringes only on the left of each window.
- · `menu-bar-showhide-fringe-menu-customize-reset` Reset the fringe mode: display fringes on both sides of a window.
- · `menu-bar-showhide-fringe-menu-customize-right` Display fringes only on the right of each window.
- · `menu-bar-showhide-tool-bar-menu-customize-disable` Do not display tool bars.
- · `menu-bar-showhide-tool-bar-menu-customize-enable-bottom` Display tool bars on the bottom side.
- · `menu-bar-showhide-tool-bar-menu-customize-enable-left` Display tool bars on the left side.
- · `menu-bar-showhide-tool-bar-menu-customize-enable-right` Display tool bars on the right side.
- · `menu-bar-showhide-tool-bar-menu-customize-enable-top` Display tool bars on the top side.
- · `menu-bar--toggle-truncate-long-lines` Toggle long lines mode.
- · `menu-bar--visual-line-mode-enable` Enable visual line mode.
- · `menu-bar-window-divider-customize` Show customization buffer for `window-divider' group.
- · `menu-bar--wrap-long-lines-window-edge` Wrap long lines at window edge.
- · `menu-find-file-existing` Edit the existing file FILENAME.
- · `menu-set-font` Interactively select a font and make it the default on all frames.

## mercury

- · `mercury-mode` Major mode for editing Mercury programs.

## merge

- · `merge-frames` Merge the main window of FRAME2 into FRAME1.
- · `merge-tabs` Merge the main window of TAB2 into TAB1.

## message

- · `message-bold-region` Bold all nonblank characters in the region.
- · `message-bounce` Re-mail the current message.
- · `message-cancel-news` Cancel an article you posted.
- · `message-followup` Follow up to the message in the current buffer.
- · `message-forward` Forward the current message via mail.
- · `message-goto-eoh` 
- · `message-insinuate-rmail` Let RMAIL use message to forward.
- · `message-mail` Start editing a mail message to be sent.
- · `message-mail-other-frame` Like `message-mail' command, but display mail buffer in another frame.
- · `message-mail-other-window` Like `message-mail' command, but display mail buffer in another window.
- · `message-mailto` Command to parse command line mailto: links.
- · `message-mode` Major mode for editing mail and news to be sent.
- · `message-news` Start editing a news article to be sent.
- · `message-news-other-frame` Start editing a news article to be sent.
- · `message-news-other-window` Start editing a news article to be sent.
- · `message-recover` Reread contents of current buffer from its last auto-save file.
- · `message-reply` Start editing a reply to the article in the current buffer.
- · `message-resend` Resend the current article to ADDRESS.
- · `message-supersede` Start composing a message to supersede the current message.
- · `message-unbold-region` Remove all boldness (overstruck characters) in the region.
- · `message-wide-reply` Make a "wide" reply to the message in the current buffer.

## messages

- · `messages-buffer-mode` Major mode used in the "*Messages*" buffer.

## metafont

- · `metafont-mode` Major mode for editing Metafont sources.

## metapost

- · `metapost-mode` Major mode for editing MetaPost sources.

## mh

- · `mh-folder-mode` Major MH-E mode for "editing" an MH folder scan listing.
- · `mh-fully-kill-draft` Quit editing and delete draft message.
- · `mh-nmail` Check for new mail in inbox folder.
- · `mh-rmail` Incorporate new mail with MH.
- · `mh-send-letter` Save draft and send message.
- · `mh-smail` Compose a message with the MH mail system.
- · `mh-smail-other-window` Compose a message with the MH mail system in other window.
- · `mh-version` Display version information about MH-E and the MH mail handling system.

## mhtml

- · `mhtml-mode` Major mode based on `html-mode', but works with embedded JS and CSS.
- · `mhtml-ts-mode` Major mode for editing HTML with embedded JavaScript and CSS.

## midnight

- · `midnight-mode` Non-nil means run `midnight-hook' at midnight.

## minibuffer

- · `minibuffer-beginning-of-buffer` Move to the logical beginning of the minibuffer.
- · `minibuffer-choose-completion` Run `choose-completion' from the minibuffer in its completions window.
- · `minibuffer-choose-completion-or-exit` Choose the completion from the minibuffer or exit the minibuffer.
- · `minibuffer-complete` Complete the minibuffer contents as far as possible.
- · `minibuffer-complete-and-exit` Exit if the minibuffer contains a valid completion.
- · `minibuffer-complete-defaults` Complete as far as possible using the minibuffer defaults.
- · `minibuffer-complete-history` Complete as far as possible using the minibuffer history.
- · `minibuffer-complete-word` Complete the minibuffer contents at most a single word.
- · `minibuffer-completion-exit` Call `exit-minibuffer', inserting the selected completion first if any.
- · `minibuffer-completion-help` Display a list of possible completions of the current minibuffer contents.
- · `minibuffer-depth-indicate-mode` Toggle Minibuffer Depth Indication mode.
- · `minibuffer-electric-default-mode` Toggle Minibuffer Electric Default mode.
- · `minibuffer-force-complete` Complete the minibuffer to an exact match.
- · `minibuffer-force-complete-and-exit` Complete the minibuffer with first of the matches and exit.
- · `minibuffer-hide-completions` Get rid of an out-of-date *Completions* buffer.
- · `minibuffer-inactive-mode` Major mode to use in the minibuffer when it is not active.
- · `minibuffer-insert-file-name-at-point` Get a file name at point in original buffer and insert it to minibuffer.
- · `minibuffer-next-column-completion` Move to the next completion column from the minibuffer.
- · `minibuffer-next-completion` Move to the next item in its completions window from the minibuffer.
- · `minibuffer-next-line-completion` Move to the next completion line from the minibuffer.
- · `minibuffer-nonselected-mode` Minor mode to warn about non-selected active minibuffer window.
- · `minibuffer-previous-column-completion` Move to the previous completion column from the minibuffer.
- · `minibuffer-previous-completion` Move to the previous item in its completions window from the minibuffer.
- · `minibuffer-previous-line-completion` Move to the previous completion line from the minibuffer.
- · `minibuffer-recenter-top-bottom` Run `recenter-top-bottom' from the minibuffer in its original window.
- · `minibuffer-regexp-mode` Minor mode for editing regular expressions in the minibuffer.
- · `minibuffer-scroll-down-command` Run `scroll-down-command' from the minibuffer in its original window.
- · `minibuffer-scroll-other-window` Run `scroll-other-window' from the minibuffer in its original window.
- · `minibuffer-scroll-other-window-down` Run `scroll-other-window-down' from the minibuffer in its original window.
- · `minibuffer-scroll-up-command` Run `scroll-up-command' from the minibuffer in its original window.

## minimize

- · `minimize-window` Minimize WINDOW.

## minor

- · `minor-mode-menu-from-indicator` Show menu for minor mode specified by INDICATOR.

## mixal

- · `mixal-mode` Major mode for the mixal asm language.

## mkdir

- · `mkdir` Create the directory DIR and optionally any nonexistent parent dirs.

## mml

- · `mml-attach-file` Attach a file to the outgoing MIME message.

## mode

- · `mode-line-bury-buffer` Like `bury-buffer', but temporarily select EVENT's window.
- · `mode-line-change-eol` Cycle through the various possible kinds of end-of-line styles.
- · `mode-line-invisible-mode` Toggle the mode-line visibility of the current buffer.
- · `mode-line-minor-mode-help` Describe minor mode for EVENT on minor modes area of the mode line.
- · `mode-line-next-buffer` Like `next-buffer', but temporarily select EVENT's window.
- · `mode-line-other-buffer` Switch to the most recently selected buffer other than the current one.
- · `mode-line-previous-buffer` Like `previous-buffer', but temporarily select EVENT's window.
- · `mode-line-toggle-modified` Toggle the buffer-modified flag from the mode-line.
- · `mode-line-toggle-read-only` Like toggling `read-only-mode', for the mode-line.
- · `mode-line-unbury-buffer` Call `unbury-buffer' in this window.
- · `mode-line-widen` Widen a buffer from the mode-line.

## modifier

- · `modifier-bar-mode` Toggle display of the key-modifier tool bar.

## modify

- · `modify-face` Modify attributes of faces interactively.
- · `modify-syntax-entry` Set syntax for character CHAR according to string NEWENTRY.

## modula

- · `modula-2-mode` This is a mode intended to support program development in Modula-2.

## morse

- · `morse-region` Convert plain text in region to Morse code.

## mouse

- · `mouse-appearance-menu` Show a menu for changing the default face in the current buffer.
- · `mouse-avoidance-mode` Set Mouse Avoidance mode to MODE.
- · `mouse-buffer-menu` Pop up a menu of buffers for selection with the mouse.
- · `mouse-delete-other-windows` Delete all windows except the one you click on.
- · `mouse-delete-window` Delete the window you click on.
- · `mouse-drag-and-drop-region` Move text in the region to point where mouse is dragged to.
- · `mouse-drag-bottom-edge` Drag bottom edge of a frame with the mouse.
- · `mouse-drag-bottom-left-corner` Drag bottom left corner of a frame with the mouse.
- · `mouse-drag-bottom-right-corner` Drag bottom right corner of a frame with the mouse.
- · `mouse-drag-drag` "Drag" the page according to a mouse drag.
- · `mouse-drag-header-line` Change the height of a window by dragging on its header line.
- · `mouse-drag-left-edge` Drag left edge of a frame with the mouse.
- · `mouse-drag-mode-line` Change the height of a window by dragging on its mode line.
- · `mouse-drag-region` Set the region to the text that the mouse is dragged over.
- · `mouse-drag-region-rectangle` Set the region to the rectangle that the mouse is dragged over.
- · `mouse-drag-region-shift-adjust` Adjust the already active region while dragging the mouse.
- · `mouse-drag-right-edge` Drag right edge of a frame with the mouse.
- · `mouse-drag-secondary` Set the secondary selection to the text that the mouse is dragged over.
- · `mouse-drag-tab-line` Drag frame with tab line in its topmost window.
- · `mouse-drag-throw` "Throw" the page according to a mouse drag.
- · `mouse-drag-top-edge` Drag top edge of a frame with the mouse.
- · `mouse-drag-top-left-corner` Drag top left corner of a frame with the mouse.
- · `mouse-drag-top-right-corner` Drag top right corner of a frame with the mouse.
- · `mouse-drag-vertical-line` Change the width of a window by dragging on a vertical line.
- · `mouse-kill` Kill the region between point and the mouse click.
- · `mouse-kill-ring-save` Copy the region between point and the mouse click in the kill ring.
- · `mouse-kill-secondary` Kill the text in the secondary selection.
- · `mouse-minor-mode-menu` Show minor-mode menu for EVENT on minor modes area of the mode line.
- · `mouse-save-then-kill` Set the region according to CLICK; the second time, kill it.
- · `mouse-secondary-save-then-kill` Set the secondary selection and save it to the kill ring.
- · `mouse-select-font` Prompt for a font name, using `x-popup-menu', and return it.
- · `mouse-select-window` Select the window clicked on; don't move point.
- · `mouse-set-font` Set the default font for the selected frame.
- · `mouse-set-mark` Set mark at the position clicked on with the mouse.
- · `mouse-set-point` Move point to the position clicked on with the mouse.
- · `mouse-set-region` Set the region to the text dragged over, and copy to kill ring.
- · `mouse-set-secondary` Set the secondary selection to the text that the mouse is dragged over.
- · `mouse-shift-adjust-mode` Toggle mouse shift adjustment mode.
- · `mouse-split-window-horizontally` Select Emacs window mouse is on, then split it horizontally in half.
- · `mouse-split-window-vertically` Select Emacs window mouse is on, then split it vertically in half.
- · `mouse-start-secondary` Set one end of the secondary selection to the position clicked on.
- · `mouse-tear-off-window` Delete the selected window, and create a new frame displaying its buffer.
- · `mouse-wheel-global-text-scale` Increase or decrease the global font size according to the EVENT.
- · `mouse-wheel-mode` Toggle mouse wheel support (Mouse Wheel mode).
- · `mouse-wheel-text-scale` Adjust font size of the default face according to EVENT.
- · `mouse-yank-at-click` Insert the last stretch of killed text at the position clicked on.
- · `mouse-yank-primary` Insert the primary selection,
- · `mouse-yank-secondary` Insert the secondary selection at the position clicked on.

## move

- ✓ `move-beginning-of-line` Move point to visible beginning of current logical line.
- ✓ `move-end-of-line` Move point to end of current line as displayed.
- · `move-file-to-trash` Move the file (or directory) named FILENAME to the trash.
- ✓ `move-past-close-and-reindent` Move past next `)', delete indentation before it, then indent after it.
- ✓ `move-to-column` Move point to column COLUMN in the current line.
- · `move-to-left-margin` Move to the left margin of the current line.
- · `move-to-tab-stop` Move point to next defined tab-stop column.
- · `move-to-window-line` Position point relative to window.
- ✓ `move-to-window-line-top-bottom` Position point relative to window.

## mpc

- · `mpc` Main entry point for MPC.

## mpuz

- · `mpuz` Multiplication puzzle with GNU Emacs.

## msb

- · `msb-mode` Toggle Msb mode.

## mspools

- · `mspools-show` Show the list of non-empty spool files in the *spools* buffer.

## mule

- · `mule-diag` Display diagnosis of the multilingual environment (Mule).

## multi

- · `multi-file-replace-regexp-as-diff` Show as diffs replacements of REGEXP with TO-STRING in FILES.
- · `multi-isearch-buffers` Start multi-buffer Isearch on a list of BUFFERS.
- · `multi-isearch-buffers-regexp` Start multi-buffer regexp Isearch on a list of BUFFERS.
- · `multi-isearch-files` Start multi-buffer Isearch on a list of FILES.
- · `multi-isearch-files-regexp` Start multi-buffer regexp Isearch on a list of FILES.
- · `multi-occur` Show all lines in buffers BUFS containing a match for REGEXP.
- · `multi-occur-in-matching-buffers` Show all lines matching REGEXP in buffers specified by BUFREGEXP.

## mwheel

- · `mwheel-scroll` Scroll up or down according to the EVENT.

## name

- · `name-last-kbd-macro` Assign a name to the last keyboard macro defined.

## narrow

- ✓ `narrow-to-defun` Make text outside current defun invisible.
- ✓ `narrow-to-page` Make text outside current page invisible.
- ✓ `narrow-to-region` Restrict editing in this buffer to the current region.

## native

- · `native-compile-prune-cache` Remove *.eln files that aren't usable by the current Emacs build.

## nato

- · `nato-region` Convert plain text in region to NATO spelling alphabet.

## negative

- ✓ `negative-argument` Begin a negative numeric argument for the next command.

## netstat

- · `netstat` Run `netstat-program' and display diagnostic output.

## network

- · `network-connection` Open a network connection to HOST on PORT.
- · `network-connection-to-service` Open a network connection to SERVICE on HOST.

## newline

- ✓ `newline` Insert a newline, and move to left margin of the new line.
- ✓ `newline-and-indent` Insert a newline, then indent according to major mode.

## newsticker

- · `newsticker-plainview` Start newsticker plainview.
- · `newsticker-show-news` Start reading news.  You may want to bind this to a key.
- · `newsticker-start` Start the newsticker.
- · `newsticker-start-ticker` Start newsticker's ticker (but not the news retrieval).
- · `newsticker-treeview` Start newsticker treeview.

## next

- ✓ `next-buffer` In selected window switch to ARGth next buffer.
- · `next-column-completion` Move to the item in the next column of the completions buffer.
- · `next-complete-history-element` Get next history element that completes the minibuffer before the point.
- · `next-completion` Move according to `completions-format' to next completion item.
- ✓ `next-error` Visit next `next-error' message and corresponding source code.
- · `next-error-follow-minor-mode` Minor mode for compilation, occur and diff modes.
- · `next-error-no-select` Move point to the next error in the `next-error' buffer and highlight match.
- · `next-error-select-buffer` Select a `next-error' capable BUFFER and set it as the last used.
- · `next-error-this-buffer-no-select` Move point to the next error in the current buffer and highlight match.
- · `next-file` Select next file among files in current tags table.
- · `next-history-element` Insert into the minibuffer the Nth next element of minibuffer history.
- ✓ `next-line` Move cursor vertically down ARG lines.
- · `next-line-completion` Move to completion candidate on the next line in the completions buffer.
- · `next-line-or-history-element` Move cursor vertically down ARG lines, or to the next history element.
- · `next-logical-line` Move cursor vertically down ARG lines.
- · `next-match` Visit next `next-error' message and corresponding source code.
- · `next-matching-history-element` Find the next history element that matches REGEXP.
- · `next-multiframe-window` Select the next window, regardless of which frame it is on.
- · `next-window-any-frame` Select the next window, regardless of which frame it is on.

## nndiary

- · `nndiary-generate-nov-databases` Generate NOV databases in all nndiary directories.

## nnfolder

- · `nnfolder-generate-active-file` Look for mbox folders in the nnfolder directory and make them into groups.

## nnml

- · `nnml-generate-nov-databases` Generate NOV databases in all nnml directories.

## nonincremental

- · `nonincremental-repeat-search-backward` Search backward for the previous search string or regexp.
- · `nonincremental-repeat-search-forward` Search forward for the previous search string or regexp.
- · `nonincremental-re-search-backward` Read a regular expression and search backward for it nonincrementally.
- · `nonincremental-re-search-forward` Read a regular expression and search for it nonincrementally.
- · `nonincremental-search-backward` Read a string and search backward for it nonincrementally.
- · `nonincremental-search-forward` Read a string and search for it nonincrementally.

## normal

- · `normal-erase-is-backspace-mode` Toggle the Erase and Delete mode of the Backspace and Delete keys.
- · `normal-mode` Choose the major mode for this buffer automatically.

## not

- ✓ `not-modified` Mark current buffer as unmodified, not needing to be saved.

## nroff

- · `nroff-mode` Major mode for editing text intended for nroff to format.

## nslookup

- · `nslookup` Run `nslookup-program'.
- · `nslookup-host` Look up the DNS information for HOST (name or IP address).

## number

- ✓ `number-to-register` Store NUMBER in REGISTER.

## nxml

- · `nxml-mode` Major mode for editing XML.

## objc

- · `objc-mode` Major mode for editing Objective C code.

## occur

- ✓ `occur` Show all lines in the current buffer containing a match for REGEXP.
- · `occur-cease-edit` Switch from Occur Edit mode to Occur mode.
- · `occur-edit-mode` Major mode for editing *Occur* buffers.
- · `occur-menu-map` Menu for `occur-mode'.
- · `occur-mode` Major mode for output from \[occur].
- · `occur-mode-display-occurrence` Display in another window the occurrence the current line describes.
- · `occur-mode-goto-occurrence` Go to the occurrence specified by EVENT, a mouse click.
- · `occur-mode-goto-occurrence-other-window` Go to the occurrence the current line describes, in another window.
- · `occur-mode-mouse-goto` Go to the occurrence specified by EVENT, a mouse click.
- · `occur-next` Move to the Nth (default 1) next match in an Occur mode buffer.
- · `occur-next-error` Move to the ARGPth (default 1) next match in an Occur mode buffer.
- · `occur-prev` Move to the Nth (default 1) previous match in an Occur mode buffer.
- · `occur-rename-buffer` Rename the current *Occur* buffer to *Occur: original-buffer-name*.
- · `occur-symbol-at-mouse` Display an occur buffer for the symbol at EVENT.
- · `occur-word-at-mouse` Display an occur buffer for the word at EVENT.

## octave

- · `octave-mode` Major mode for editing Octave code.

## opascal

- · `opascal-mode` Major mode for editing OPascal code.

## open

- · `open-dribble-file` Start writing input events to a dribble file called FILE.
- ✓ `open-line` Insert a newline and leave point before it.
- ✓ `open-rectangle` Blank out the region-rectangle, shifting text right.
- · `open-termscript` Start writing all terminal output to FILE as well as the terminal.

## org

- · `org-agenda` Dispatch agenda commands to collect entries to the agenda buffer.
- · `org-agenda-list` Produce a daily/weekly view from all files in variable `org-agenda-files'.
- · `org-agenda-list-stuck-projects` Create agenda view for projects that are stuck.
- · `org-agenda-set-restriction-lock` Set restriction lock for agenda to current subtree or file.
- · `org-agenda-to-appt` Activate appointments found in `org-agenda-files'.
- · `org-babel-load-file` Load Emacs Lisp source code blocks in the Org FILE.
- · `org-calendar-goto-agenda` Compute the Org agenda for the calendar date displayed at the cursor.
- · `org-capture` Capture something.
- · `org-capture-import-remember-templates` Set `org-capture-templates' to be similar to `org-remember-templates'.
- · `org-capture-string` Capture STRING with the template selected by KEYS.
- · `org-cite-insert` Insert a citation at point.
- · `org-customize` Call the customize function with org as argument.
- · `org-cycle` TAB-action and visibility cycling for Org mode.
- · `org-cycle-agenda-files` Cycle through the files in `org-agenda-files'.
- · `org-cycle-global` Cycle the global visibility.  For details see `org-cycle'.
- · `org-decrypt-entries` Decrypt all entries in the current buffer.
- · `org-decrypt-entry` Decrypt the content of the current headline.
- · `org-encrypt-entries` Encrypt all top-level entries in the current buffer.
- · `org-encrypt-entry` Encrypt the content of the current headline.
- · `org-koma-letter-export-as-latex` Export current buffer as a KOMA Scrlttr2 letter.
- · `org-koma-letter-export-to-latex` Export current buffer as a KOMA Scrlttr2 letter (tex).
- · `org-koma-letter-export-to-pdf` Export current buffer as a KOMA Scrlttr2 letter (pdf).
- · `org-mode` Outline-based notes management and organizer, alias
- · `org-open-at-point-global` Follow a link or a timestamp like Org mode does.
- · `org-reload` Reload all Org Lisp files.
- · `org-search-view` Show all entries that contain a phrase or words or regular expressions.
- · `org-store-agenda-views` Store agenda views.
- · `org-submit-bug-report` Submit a bug report on Org via mail.
- · `org-switchb` Switch between Org buffers.
- · `org-tags-view` Show all headlines for all `org-agenda-files' matching a TAGS criterion.
- · `org-todo-list` Show all (not done) TODO entries from all agenda files in a single list.
- · `org-toggle-sticky-agenda` Toggle `org-agenda-sticky'.
- · `org-version` Show the Org version.

## other

- · `other-frame` Select the ARGth visible frame on current display, and raise it.
- · `other-frame-prefix` Display the buffer of the next command in a new frame.
- · `other-tab-prefix` Display the buffer of the next command in a new tab.
- ✓ `other-window` Select another window in cyclic ordering of windows.
- ✓ `other-window-backward` Select another window in the reverse cyclic ordering of windows.
- · `other-window-prefix` Display the buffer of the next command in a new window.

## outline

- · `outline-minor-mode` Toggle Outline minor mode.
- · `outline-mode` Set major mode for editing outlines with selective display.

## outlineify

- · `outlineify-sticky` Activate outline mode and establish file var so it is started subsequently.

## outlinify

- · `outlinify-sticky` Activate outline mode and establish file var so it is started subsequently.

## overwrite

- ✓ `overwrite-mode` Toggle Overwrite mode.

## package

- · `package-autoremove` Remove packages that are no longer needed.
- · `package-autosuggest` Prompt the user to install the suggested packages.
- · `package-autosuggest-mode` Enable automatic suggestions for installing add-on packages.
- · `package-browse-url` Open the website of the package under point in a browser.
- · `package-delete` Delete package PKG-DESC.
- · `package-import-keyring` Import keys from FILE.
- · `package-initialize` Load Emacs Lisp packages, and activate them.
- · `package-install` Install the package PKG.
- · `package-install-file` Install a package from FILE.
- · `package-install-from-buffer` Install a package from the current buffer.
- · `package-install-selected-packages` Ensure packages in `package-selected-packages' are installed.
- · `package-list-packages` Display a list of packages.
- · `package-recompile` Byte-compile package PKG again.
- · `package-recompile-all` Byte-compile all installed packages.
- · `package-refresh-contents` Download descriptions of all configured ELPA packages.
- · `package-reinstall` Reinstall package PKG.
- · `package-report-bug` Prepare a message to send to the maintainers of a package.
- · `package-upgrade` Upgrade package NAME if a newer version exists.
- · `package-upgrade-all` Refresh package list and upgrade all packages.
- · `package-vc-checkout` Clone the sources for PKG-DESC into DIRECTORY and visit that directory.
- · `package-vc-install` Fetch a package described by PACKAGE and set it up for use with Emacs.
- · `package-vc-install-from-checkout` Install the package NAME from its source directory DIR.
- · `package-vc-install-selected-packages` Ensure packages specified in `package-vc-selected-packages' are installed.
- · `package-vc-prepare-patch` Email patches for REVISIONS to maintainer of package PKG-DESC using SUBJECT.
- · `package-vc-rebuild` Rebuild the installation for package given by PKG-DESC.
- · `package-vc-upgrade` Upgrade the package described by PKG-DESC from package's VC repository.
- · `package-vc-upgrade-all` Upgrade all installed VC packages.

## paragraph

- · `paragraph-indent-minor-mode` Minor mode for editing text, with leading spaces starting a paragraph.
- · `paragraph-indent-text-mode` Major mode for editing text, with leading spaces starting a paragraph.

## pascal

- · `pascal-mode` Major mode for editing Pascal code.

## pcomplete

- · `pcomplete` Support extensible programmable completion.
- · `pcomplete-continue` Complete without reference to any cycling completions.
- · `pcomplete-expand` Expand the textual value of the current argument.
- · `pcomplete-expand-and-complete` Expand the textual value of the current argument.
- · `pcomplete-help` Display any help information relative to the current argument.
- · `pcomplete-list` Show the list of possible completions for the current argument.
- · `pcomplete-reverse` If cycling completion is in use, cycle backwards.

## pdb

- · `pdb` Run COMMAND-LINE in the `*gud-FILE*' buffer to debug Python programs.

## pending

- · `pending-delete-mode` Toggle Delete Selection mode.

## perl

- · `perl-mode` Major mode for editing Perl code.

## perldb

- · `perldb` Debug a perl program with gud.

## php

- · `php-ts-mode` Major mode for editing PHP, powered by tree-sitter.
- · `php-ts-mode-run-php-webserver` Run PHP built-in web server.

## picture

- · `picture-mode` Switch to Picture mode, in which a quarter-plane screen model is used.

## pike

- · `pike-mode` Major mode for editing Pike code.

## ping

- · `ping` Ping HOST using `ping-program'.

## pixel

- · `pixel-scroll-interpolate-down` Interpolate a scroll downwards by one page.
- · `pixel-scroll-interpolate-up` Interpolate a scroll upwards by one page.
- · `pixel-scroll-mode` A minor mode to scroll text pixel-by-pixel.
- · `pixel-scroll-precision-mode` Toggle pixel scrolling.

## plain

- · `plain-tex-mode` Major mode for editing files of input for plain TeX.
- · `plain-TeX-mode` Major mode for editing files of input for plain TeX.

## play

- · `play-sound-file` Play sound stored in FILE.

## plstore

- · `plstore-mode` Major mode for editing plstore files.

## point

- ✓ `point-to-register` Store current location of point in REGISTER.

## pong

- · `pong` Play pong and waste time.

## pop

- · `pop-global-mark` Pop off global mark ring and jump to the top location.
- · `pop-tag-mark` Go back to the previous position in xref history.
- · `pop-to-buffer` Display buffer specified by BUFFER-OR-NAME and select its window.
- · `pop-to-mark-command` Jump to mark, and pop a new position for mark off the ring.

## posix

- · `posix-search-backward` Search backward from point for match for REGEXP according to Posix rules.
- · `posix-search-forward` Search forward from point for REGEXP according to Posix rules.

## pp

- · `pp-buffer` Prettify the current buffer with printed representation of a Lisp object.
- · `pp-eval-expression` Evaluate EXPRESSION and pretty-print its value.
- · `pp-eval-last-sexp` Run `pp-eval-expression' on sexp before point.
- · `pp-macroexpand-expression` Macroexpand EXPRESSION and pretty-print its value.
- · `pp-macroexpand-last-sexp` Run `pp-macroexpand-expression' on sexp before point.

## pr

- · `pr-customize` Customization of the `printing' group.
- · `pr-despool-preview` Preview spooled PostScript.
- · `pr-despool-print` Send the spooled PostScript to the printer.
- · `pr-despool-ps-print` Send the spooled PostScript to the printer or use ghostscript to print it.
- · `pr-despool-using-ghostscript` Print spooled PostScript using ghostscript.
- · `pr-help` Help for the printing package.
- · `pr-interface` Activate the printing interface buffer.
- · `pr-printify-buffer` Replace nonprinting characters in buffer with printable representations.
- · `pr-printify-directory` Replace nonprinting characters in directory with printable representations.
- · `pr-printify-region` Replace nonprinting characters in region with printable representations.
- · `pr-ps-buffer-preview` Preview buffer using ghostview.
- · `pr-ps-buffer-print` Print buffer using PostScript printer.
- · `pr-ps-buffer-ps-print` Print buffer using PostScript printer or through ghostscript.
- · `pr-ps-buffer-using-ghostscript` Print buffer using PostScript through ghostscript.
- · `pr-ps-directory-preview` Preview directory using ghostview.
- · `pr-ps-directory-print` Print directory using PostScript printer.
- · `pr-ps-directory-ps-print` Print directory using PostScript printer or through ghostscript.
- · `pr-ps-directory-using-ghostscript` Print directory using PostScript through ghostscript.
- · `pr-ps-fast-fire` Fast fire function for PostScript printing.
- · `pr-ps-file-preview` Preview PostScript file FILENAME.
- · `pr-ps-file-print` Print PostScript file FILENAME.
- · `pr-ps-file-ps-print` Send PostScript file FILENAME to printer or use ghostscript to print it.
- · `pr-ps-file-up-preview` Preview PostScript file FILENAME.
- · `pr-ps-file-up-ps-print` Process a PostScript file IFILENAME and send it to printer.
- · `pr-ps-file-using-ghostscript` Print PostScript file FILENAME using ghostscript.
- · `pr-ps-mode-preview` Preview major mode using ghostview.
- · `pr-ps-mode-print` Print major mode using PostScript printer.
- · `pr-ps-mode-ps-print` Print major mode using PostScript or through ghostscript.
- · `pr-ps-mode-using-ghostscript` Print major mode using PostScript through ghostscript.
- · `pr-ps-name` Interactively select a PostScript printer.
- · `pr-ps-region-preview` Preview region using ghostview.
- · `pr-ps-region-print` Print region using PostScript printer.
- · `pr-ps-region-ps-print` Print region using PostScript printer or through ghostscript.
- · `pr-ps-region-using-ghostscript` Print region using PostScript through ghostscript.
- · `pr-ps-utility` Interactively select a PostScript utility.
- · `pr-show-lpr-setup` Show current lpr settings.
- · `pr-show-pr-setup` Show current printing settings.
- · `pr-show-ps-setup` Show current ps-print settings.
- · `pr-toggle-duplex` Toggle duplex.
- · `pr-toggle-faces` Toggle printing with faces.
- · `pr-toggle-file-duplex` Toggle duplex for PostScript file.
- · `pr-toggle-file-landscape` Toggle landscape for PostScript file.
- · `pr-toggle-file-tumble` Toggle tumble for PostScript file.
- · `pr-toggle-ghostscript` Toggle printing using ghostscript.
- · `pr-toggle-header` Toggle printing header.
- · `pr-toggle-header-frame` Toggle printing header frame.
- · `pr-toggle-landscape` Toggle landscape.
- · `pr-toggle-line` Toggle line number.
- · `pr-toggle-lock` Toggle menu lock.
- · `pr-toggle-mode` Toggle auto mode.
- · `pr-toggle-region` Toggle whether the region is automagically detected.
- · `pr-toggle-spool` Toggle spooling.
- · `pr-toggle-tumble` Toggle tumble.
- · `pr-toggle-upside-down` Toggle upside-down.
- · `pr-toggle-zebra` Toggle zebra stripes.
- · `pr-txt-buffer` Print buffer using text printer.
- · `pr-txt-directory` Print directory using text printer.
- · `pr-txt-fast-fire` Fast fire function for text printing.
- · `pr-txt-mode` Print major mode using text printer.
- · `pr-txt-name` Interactively select a text printer.
- · `pr-txt-region` Print region using text printer.

## prefer

- · `prefer-coding-system` Add CODING-SYSTEM at the front of the priority list for automatic detection.

## prepare

- · `prepare-user-lisp` Byte-compile, scrape autoloads and prepare files in `user-lisp-directory'.

## prepend

- · `prepend-to-buffer` Prepend to specified BUFFER the text of the region.
- ✓ `prepend-to-register` Prepend region of text between START and END to REGISTER.

## prettify

- · `prettify-special-glyphs-mode` Mode to display pretty special character glyphs.
- · `prettify-symbols-mode` Toggle Prettify Symbols mode.

## previous

- ✓ `previous-buffer` In selected window switch to ARGth previous buffer.
- · `previous-column-completion` Move to the item in the previous column of the completions buffer.
- · `previous-complete-history-element` Get previous history element that completes the minibuffer before the point.
- · `previous-completion` Move according to `completions-format' to previous completion item.
- ✓ `previous-error` Visit previous `next-error' message and corresponding source code.
- · `previous-error-no-select` Move point to the previous error in the `next-error' buffer and highlight match.
- · `previous-error-this-buffer-no-select` Move point to the previous error in the current buffer and highlight match.
- · `previous-history-element` Insert into the minibuffer the Nth previous element of minibuffer history.
- ✓ `previous-line` Move cursor vertically up ARG lines.
- · `previous-line-completion` Move to completion candidate on the previous line in the completions buffer.
- · `previous-line-or-history-element` Move cursor vertically up ARG lines, or to the previous history element.
- · `previous-logical-line` Move cursor vertically up ARG lines.
- · `previous-matching-history-element` Find the previous history element that matches REGEXP.
- · `previous-multiframe-window` Select the previous window, regardless of which frame it is on.
- · `previous-window-any-frame` Select the previous window, regardless of which frame it is on.

## print

- · `print-buffer` Paginate and print buffer contents.
- · `print-region` Paginate and print the region contents.

## proced

- · `proced` Generate a listing of system processes and their attributes.

## process

- · `process-menu-delete-process` Kill process at point in a `list-processes' buffer.
- · `process-menu-mode` Major mode for listing the processes called by Emacs.

## profiler

- · `profiler-find-profile` Open profile FILENAME.
- · `profiler-find-profile-other-frame` Open profile FILENAME.
- · `profiler-find-profile-other-window` Open profile FILENAME.
- · `profiler-start` Start/restart profilers.

## prog

- ✓ `prog-fill-reindent-defun` Refill or reindent the paragraph or defun that contains point.
- · `prog-fill-reindent-defun-default` Default implementation of `prog-fill-reindent-defun'.
- · `prog-indent-sexp` Indent the expression after point.
- · `prog-mode` Major mode for editing programming language source code.

## project

- ✓ `project-any-command` Run the next command in the current project.
- ✓ `project-async-shell-command` Run `async-shell-command' in the current project's root directory.
- ✓ `project-compile` Run `compile' in the project root.
- · `project-customize-dirlocals` Run `customize-dirlocals' in current project's root.
- ✓ `project-dired` Start Dired in the current project's root.
- · `project-display-buffer` Display BUFFER-OR-NAME in some window, without selecting it.
- · `project-display-buffer-other-frame` Display BUFFER-OR-NAME preferably in another frame.
- ✓ `project-eshell` Start Eshell in the current project's root directory.
- ✓ `project-execute-extended-command` Execute an extended command in project root.
- ✓ `project-find-dir` Start Dired in a directory inside the current project.
- ✓ `project-find-file` Visit a file (with completion) in the current project.
- · `project-find-matching-buffer` Switch to a matching buffer in another project.
- ✓ `project-find-regexp` Find all matches for REGEXP in the current project's roots.
- · `project-forget-project` Remove directory PROJECT-ROOT from the project list.
- ✓ `project-kill-buffers` Kill the buffers belonging to the current project.
- ✓ `project-list-buffers` Display a list of project buffers.
- ✓ `project-or-external-find-file` Visit a file (with completion) in the current project or external roots.
- ✓ `project-or-external-find-regexp` Find all matches for REGEXP in the project roots or external roots.
- · `project-other-frame-command` Run project command, displaying resultant buffer in another frame.
- · `project-other-tab-command` Run project command, displaying resultant buffer in a new tab.
- · `project-other-window-command` Run project command, displaying resultant buffer in another window.
- · `project-prefix-or-any-command` Run the next command in the current project.
- · `project-query-replace-regexp` Query-replace REGEXP in all the files of the project.
- · `project-recompile` Run `recompile' in the project root with an appropriate buffer.
- · `project-remember-project` Add project PR to the front of the project list.
- · `project-root-find-file` Edit file FILENAME.
- ✓ `project-save-some-buffers` Like `save-some-buffers', but only for this project's buffers.
- · `project-search` Search for REGEXP in all the files of the project.
- ✓ `project-shell` Start an inferior shell in the current project's root directory.
- ✓ `project-shell-command` Run `shell-command' in the current project's root directory.
- ✓ `project-switch-project` "Switch" to another project by running an Emacs command.
- ✓ `project-switch-to-buffer` Display buffer BUFFER-OR-NAME in the selected window.
- ✓ `project-vc-dir` Run VC-Dir in the current project's root.

## prolog

- · `prolog-mode` Major mode for editing Prolog code.

## ps

- · `ps-despool` Send the spooled PostScript to the printer.
- · `ps-line-lengths` Display the correspondence between a line length and a font size.
- · `ps-mode` Major mode for editing PostScript with GNU Emacs.
- · `ps-nb-pages-buffer` Display number of pages to print this buffer, for various font heights.
- · `ps-nb-pages-region` Display number of pages to print the region, for various font heights.
- · `ps-print-buffer` Generate and print a PostScript image of the buffer.
- · `ps-print-buffer-with-faces` Generate and print a PostScript image of the buffer.
- · `ps-print-customize` Customization of ps-print group.
- · `ps-print-region` Generate and print a PostScript image of the region.
- · `ps-print-region-with-faces` Generate and print a PostScript image of the region.
- · `ps-spool-buffer` Generate and spool a PostScript image of the buffer.
- · `ps-spool-buffer-with-faces` Generate and spool a PostScript image of the buffer.
- · `ps-spool-region` Generate a PostScript image of the region and spool locally.
- · `ps-spool-region-with-faces` Generate a PostScript image of the region and spool locally.

## push

- · `push-button` Perform the action specified by a button at location POS.
- · `push-mark-command` Set mark at where point is.

## pwd

- · `pwd` Show the current default directory.

## python

- · `python-add-import` Add an import statement to the current buffer.
- · `python-base-mode` Generic major mode for editing Python files.
- · `python-fix-imports` Add missing imports and remove unused ones from the current buffer.
- · `python-import-symbol-at-point` Add an import statement for the symbol at point to the current buffer.
- · `python-mode` Major mode for editing Python files.
- · `python-remove-import` Remove an import statement from the current buffer.
- · `python-sort-imports` Sort Python imports in the current buffer.
- · `python-ts-mode` Major mode for editing Python files, using tree-sitter library.

## quail

- · `quail-set-keyboard-layout` Set the current keyboard layout to the same as keyboard KBD-TYPE.
- · `quail-show-keyboard-layout` Show the physical layout of the keyboard type KEYBOARD-TYPE.
- · `quail-update-leim-list-file` Update entries for Quail packages in `LEIM' list file in directory DIRNAME.

## query

- ✓ `query-replace` Replace some occurrences of FROM-STRING with TO-STRING.
- · `query-replace-read-transpose-from-to` Transpose the FROM and TO arguments of a `query-replace' operation.
- ✓ `query-replace-regexp` Replace some things after point matching REGEXP with TO-STRING.

## quick

- · `quick-calc` Do a quick calculation in the minibuffer without invoking full Calculator.

## quit

- ✓ `quit-window` Quit WINDOW and bury its buffer.
- · `quit-windows-on` Quit all windows showing BUFFER-OR-NAME.

## quoted

- ✓ `quoted-insert` Read next input character and insert it.
- · `quoted-printable-decode-region` Decode quoted-printable in the region between FROM and TO, per RFC 2045.

## raise

- · `raise-frame` Bring FRAME to the front, so it occludes any frames it overlaps.
- · `raise-sexp` Raise N sexps one level higher up the tree.

## rcirc

- · `rcirc` Connect to all servers in `rcirc-server-alist'.
- · `rcirc-track-minor-mode` Global minor mode for tracking activity in rcirc buffers.

## re

- · `re-builder` Construct a regexp interactively.
- · `re-search-backward` Search backward from point for regular expression REGEXP.
- · `re-search-forward` Search forward from point for regular expression REGEXP.

## read

- · `read-abbrev-file` Read abbrev definitions from file written with `write-abbrev-file'.
- · `read-char-from-minibuffer-insert-char` Insert the character you type into the minibuffer and exit minibuffer.
- · `read-char-from-minibuffer-insert-other` Reject a disallowed character typed into the minibuffer.
- · `read-color` Read a color name or RGB triplet, return a string, the color name or RGB.
- · `read-expression-switch-to-completions` Select the completion list window while reading an expression.
- · `read--expression-try-read` Try to read an Emacs Lisp expression in the minibuffer.
- · `read-extended-command-mode` Minor mode used for completion in `read-extended-command'.
- · `read-kbd-macro` Read the region as a keyboard macro definition.
- ✓ `read-only-mode` Change whether the current buffer is read-only.
- · `read-regexp-toggle-case-fold` 

## recenter

- · `recenter` Center point in selected window and maybe redisplay frame.
- · `recenter-current-error` Recenter the current displayed error in the `next-error' buffer.
- ✓ `recenter-other-window` Call `recenter-top-bottom' in the other window.
- ✓ `recenter-top-bottom` Scroll the window so that current line is in the middle of the window.

## recentf

- · `recentf` Prompt for FILE in `recentf-list' and visit it.
- · `recentf-mode` Toggle keeping track of opened files (Recentf mode).
- · `recentf-open` Prompt for FILE in `recentf-list' and visit it.

## recode

- · `recode-file-name` Change the encoding of FILE's name from CODING to NEW-CODING.
- · `recode-region` Re-decode the region (previously decoded by CODING) by NEW-CODING.

## recompile

- ✓ `recompile` Re-compile the program including the current buffer.

## recover

- · `recover-file` Visit file FILE, but get contents from its last auto-save file.
- · `recover-session` Recover auto save files from a previous Emacs session.
- · `recover-session-finish` Choose one saved session to recover auto-save files from.
- · `recover-this-file` Recover the visited file--get contents from its last auto-save file.

## rectangle

- · `rectangle-mark-mode` Toggle the region as rectangular.
- ✓ `rectangle-number-lines` Insert numbers in front of the region-rectangle.

## recursive

- · `recursive-edit` Invoke the editor command loop recursively.

## redirect

- · `redirect-debugging-output` Redirect debugging output (stderr stream) to file FILE.

## redraw

- · `redraw-display` Clear and redisplay all visible frames.

## refill

- · `refill-mode` Toggle automatic refilling (Refill mode).

## reftex

- · `reftex-citation` 
- · `reftex-index-phrases-mode` 
- · `reftex-isearch-minor-mode` 
- · `reftex-mode` Minor mode with distinct support for \label, \ref and \cite in LaTeX.

## regexp

- · `regexp-builder` Construct a regexp interactively.

## register

- · `register-preview-next` Go to next line in the register preview buffer.
- · `register-preview-previous` Go to previous line in the register preview buffer.
- · `register-to-point` Go to location stored in REGISTER, or restore configuration stored there.

## reindent

- · `reindent-then-newline-and-indent` Reindent current line, insert newline, then indent the new line.

## remember

- · `remember` Remember an arbitrary piece of data.
- · `remember-clipboard` Remember the contents of the current clipboard.
- · `remember-notes` Return the notes buffer, creating it if needed, and maybe switch to it.
- · `remember-other-frame` Call `remember' in another frame.

## remove

- · `remove-hook` Remove FUNCTION from HOOK's functions.

## rename

- ✓ `rename-buffer` Change current buffer's name to NEWNAME (a string).
- · `rename-file` Rename FILE as NEWNAME.  Both args must be strings.
- · `rename-uniquely` Rename current buffer to a similar name not already taken.
- · `rename-visited-file` Rename the file visited by the current buffer to NEW-LOCATION.

## repeat

- ✓ `repeat` Repeat most recently executed command.
- · `repeat-complex-command` Edit and re-evaluate last complex command, or ARGth from last.
- · `repeat-exit` Exit the repeating sequence.
- · `repeat-matching-complex-command` Edit and re-evaluate complex command with name matching PATTERN.
- · `repeat-mode` Toggle Repeat mode.

## replace

- · `replace-buffer-contents` Replace accessible portion of current buffer with that of SOURCE.
- · `replace-buffer-in-windows` Replace BUFFER-OR-NAME with some other buffer in all windows showing it.
- · `replace-rectangle` Replace rectangle contents with STRING on each line.
- ✓ `replace-regexp` Replace things after point matching REGEXP with TO-STRING.
- · `replace-regexp-as-diff` Show as diffs replacements of REGEXP with TO-STRING in the current buffer.
- · `replace-region-contents` Replace the region between BEG and END with that of SOURCE.
- ✓ `replace-string` Replace occurrences of FROM-STRING with TO-STRING.

## report

- · `report-emacs-bug` Report a bug in GNU Emacs.

## reposition

- · `reposition-window` Make the current definition and/or comment visible.

## repunctuate

- · `repunctuate-sentences` Put two spaces at the end of sentences from point to the end of buffer.

## reset

- · `reset-language-environment` Reset multilingual environment of Emacs to the default status.

## restart

- · `restart-emacs` Kill the current Emacs process and start a new one.

## reveal

- · `reveal-mode` Toggle uncloaking of invisible text near point (Reveal mode).

## reverse

- · `reverse-region` Reverse the order of lines in a region.

## revert

- ✓ `revert-buffer` Replace current buffer text with the text of the visited file on disk.
- ✓ `revert-buffer-quick` Like `revert-buffer', but asks for less confirmation.
- · `revert-buffer-with-coding-system` Visit the current buffer's file again using coding system CODING-SYSTEM.
- · `revert-buffer-with-fine-grain` Revert buffer preserving markers, overlays, etc.

## rgrep

- · `rgrep` Recursively grep for REGEXP in FILES in directory tree rooted at DIR.

## right

- ✓ `right-char` Move point N characters to the right (to the left if N is negative).
- ✓ `right-word` Move point N words to the right (to the left if N is negative).

## rmail

- · `rmail` Read and edit incoming mail.
- · `rmail-add-label` Add LABEL to labels associated with current RMAIL message.
- · `rmail-edit-current-message` Edit the contents of this message.
- · `rmail-input` Run Rmail on file FILENAME.
- · `rmail-kill-label` Remove LABEL from labels associated with current RMAIL message.
- · `rmail-mime` Toggle the display of a MIME message.
- · `rmail-mode` Rmail Mode is used by \<rmail-mode-map>\[rmail] for editing Rmail files.
- · `rmail-next-labeled-message` Show next message with one of the labels LABELS.
- · `rmail-output` Append this message to mail file FILE-NAME.
- · `rmail-output-as-seen` Append this message to mbox file named FILE-NAME.
- · `rmail-output-body-to-file` Write this message body to the file FILE-NAME.
- · `rmail-previous-labeled-message` Show previous message with one of the labels LABELS.
- · `rmail-set-remote-password` Set PASSWORD to be used for retrieving mail from a POP or IMAP server.
- · `rmail-sort-by-author` Sort messages of current Rmail buffer by author.
- · `rmail-sort-by-correspondent` Sort messages of current Rmail buffer by other correspondent.
- · `rmail-sort-by-date` Sort messages of current Rmail buffer by "Date" header.
- · `rmail-sort-by-labels` Sort messages of current Rmail buffer by labels.
- · `rmail-sort-by-lines` Sort messages of current Rmail buffer by the number of lines.
- · `rmail-sort-by-recipient` Sort messages of current Rmail buffer by recipient.
- · `rmail-sort-by-subject` Sort messages of current Rmail buffer by "Subject" header.
- · `rmail-summary` Display a summary of all messages, one line per message.
- · `rmail-summary-by-labels` Display a summary of all messages with one or more LABELS.
- · `rmail-summary-by-recipients` Display a summary of all messages with the given RECIPIENTS.
- · `rmail-summary-by-regexp` Display a summary of all messages according to regexp REGEXP.
- · `rmail-summary-by-senders` Display a summary of all messages whose "From" field matches SENDERS.
- · `rmail-summary-by-thread` Display a summary of messages in the same discussion thread as MSGNUM.
- · `rmail-summary-by-topic` Display a summary of all messages with the given SUBJECT.

## rng

- · `rng-nxml-mode-init` Initialize `nxml-mode' to take advantage of `rng-validate-mode'.
- · `rng-validate-mode` Minor mode performing continual validation against a RELAX NG schema.

## rot13

- · `rot13-other-window` Display current buffer in ROT13 in another window.
- · `rot13-region` ROT13 encrypt the region between START and END in current buffer.

## rotate

- ✓ `rotate-windows` Rotate child windows of WINDOW in cyclic ordering.
- ✓ `rotate-windows-back` Rotate child windows of WINDOW backwards in cyclic ordering.
- · `rotate-yank-pointer` Rotate the yanking point in the kill ring.

## route

- · `route` Run `route-program' and display diagnostic output.

## rsh

- · `rsh` Open a network login connection to host named HOST (a string).

## rst

- · `rst-minor-mode` Toggle ReST minor mode.
- · `rst-mode` Major mode for editing reStructuredText documents.

## ruby

- · `ruby-base-mode` Generic major mode for editing Ruby.
- · `ruby-mode` Major mode for editing Ruby code.
- · `ruby-ts-mode` Major mode for editing Ruby, powered by tree-sitter.

## ruler

- · `ruler-mode` Toggle display of ruler in header line (Ruler mode).

## run

- · `run-at-time` Perform an action at time TIME.
- · `run-dig` Look up DNS information for HOST (name or IP address).
- · `run-lisp` Run an inferior Lisp process, input and output via buffer `*inferior-lisp*'.
- · `run-lua` Start a Lua process named NAME, running PROGRAM.
- · `run-octave` Run an inferior Octave process, I/O via `inferior-octave-buffer'.
- · `run-php` Run an PHP interpreter as a inferior process.
- · `run-prolog` Run an inferior Prolog process, input and output via buffer *prolog*.
- · `run-python` Run an inferior Python process.
- · `run-scheme` Run an inferior Scheme process, input and output via buffer `*scheme*'.
- · `run-with-idle-timer` Perform an action the next time Emacs is idle for SECS seconds.
- · `run-with-timer` Perform an action after a delay of SECS seconds.

## rust

- · `rust-ts-mode` Major mode for editing Rust, powered by tree-sitter.

## rzgrep

- · `rzgrep` Recursively grep for REGEXP in gzipped FILES in tree rooted at DIR.

## same

- · `same-window-prefix` Display the buffer of the next command in the same window.

## save

- ✓ `save-buffer` Save current buffer in visited file if modified.
- ✓ `save-buffers-kill-emacs` Offer to save each buffer, then kill this Emacs process.
- ✓ `save-buffers-kill-terminal` Offer to save each buffer, then kill the current connection.
- · `save-place-local-mode` Toggle whether to save your place in this file between sessions.
- · `save-place-mode` Non-nil means automatically save place in each file.
- ✓ `save-some-buffers` Save some modified file-visiting buffers.  Asks user about each one.

## savehist

- · `savehist-mode` Toggle saving of minibuffer history (Savehist mode).

## scan

- · `scan-buf-next-region` Go to the start of the next region with non-nil help-echo.
- · `scan-buf-previous-region` Go to the start of the previous region with non-nil help-echo.

## scheme

- · `scheme-mode` Major mode for editing Scheme code.

## scratch

- · `scratch-buffer` Switch to the *scratch* buffer.

## scroll

- · `scroll-all-mode` Toggle shared scrolling in same-frame windows (Scroll-All mode).
- · `scroll-bar-drag` Scroll the window by dragging the scroll bar slider.
- · `scroll-bar-horizontal-drag` Scroll the window horizontally by dragging the scroll bar slider.
- · `scroll-bar-maybe-set-window-start` Set the window start according to where the scroll bar is dragged.
- · `scroll-bar-mode` Toggle vertical scroll bars on all frames (Scroll Bar mode).
- · `scroll-bar-scroll-down` Scroll the window's top line down to the location of the scroll bar click.
- · `scroll-bar-scroll-up` Scroll the line next to the scroll bar click to the top of the window.
- · `scroll-bar-set-window-start` Set the window start according to where the scroll bar is dragged.
- · `scroll-bar-toolkit-horizontal-scroll` Handle event EVENT on horizontal scroll bar.
- · `scroll-bar-toolkit-scroll` Handle event EVENT on vertical scroll bar.
- · `scroll-down` Scroll text of selected window down ARG lines.
- ✓ `scroll-down-command` Scroll text of selected window down ARG lines; or near full screen if no ARG.
- · `scroll-down-line` Scroll text of selected window down ARG lines; or one line if no ARG.
- ✓ `scroll-left` Scroll selected window display ARG columns left.
- · `scroll-lock-mode` Buffer-local minor mode for pager-like scrolling.
- ✓ `scroll-other-window` Scroll next window upward LINES lines; or near full screen if no ARG.
- ✓ `scroll-other-window-down` Scroll next window downward LINES lines; or near full screen if no ARG.
- ✓ `scroll-right` Scroll selected window display ARG columns right.
- · `scroll-up` Scroll text of selected window upward ARG lines.
- ✓ `scroll-up-command` Scroll text of selected window upward ARG lines; or near full screen if no ARG.
- · `scroll-up-line` Scroll text of selected window upward ARG lines; or one line if no ARG.

## scss

- · `scss-mode` Major mode to edit "Sassy CSS" files.

## sdb

- · `sdb` Run sdb on program FILE in buffer *gud-FILE*.

## search

- · `search-backward` Search backward from point for STRING.
- · `search-backward-regexp` Search backward from point for regular expression REGEXP.
- · `search-emacs-glossary` Display the Glossary node of the Emacs manual in Info mode.
- · `search-forward` Search forward from point for STRING.
- · `search-forward-help-for-help` Search forward in the `help-for-help' window.
- · `search-forward-regexp` Search forward from point for regular expression REGEXP.
- · `search-unencodable-char` Search forward from point for a character that is not encodable.

## secrets

- · `secrets-show-secrets` 

## select

- · `select-frame` Select FRAME.
- · `select-frame-by-id` Select the frame whose identifier is ID and raise it.
- · `select-frame-by-name` Select the frame whose name is NAME and raise it.
- · `select-tags-table` Select a tags table file from a menu of those you have already used.

## self

- · `self-insert-and-exit` Terminate minibuffer input.
- · `self-insert-command` Insert the character you type.

## semantic

- · `semantic-mode` Toggle parser features (Semantic mode).

## send

- · `send-to` Send file(s) or region text to (non-Emacs) applications or services.

## serial

- · `serial-term` Start a terminal-emulator for a serial port in a new buffer.

## server

- · `server-force-delete` Unconditionally delete connection file for server NAME.
- · `server-mode` Toggle Server mode.
- · `server-start` Allow this Emacs process to be a server for client processes.

## ses

- · `ses-mode` Major mode for Simple Emacs Spreadsheet.

## set

- · `set-background-color` Set the background color of the selected frame to COLOR-NAME.
- · `set-border-color` Set the color of the border of the selected frame to COLOR-NAME.
- · `set-buffer-file-coding-system` Set the file coding-system of the current buffer to CODING-SYSTEM.
- · `set-buffer-process-coding-system` Set coding systems for the process associated with the current buffer.
- · `set-clipboard-coding-system` Make CODING-SYSTEM used for communicating with other X clients.
- · `set-comment-column` Set the comment column based on point.
- · `set-cursor-color` Set the text cursor color of the selected frame to COLOR-NAME.
- · `set-face-background` Change the background color of face FACE to COLOR (a string).
- · `set-face-background-pixmap` Change the stipple pixmap of face FACE to STIPPLE.
- · `set-face-extend` Specify whether face FACE should be extended.
- · `set-face-font` Change font-related attributes of FACE to those of FONT.
- · `set-face-foreground` Change the foreground color of face FACE to COLOR (a string).
- · `set-face-inverse-video` Specify whether face FACE is in inverse video.
- · `set-face-inverse-video-p` Specify whether face FACE is in inverse video.
- · `set-face-stipple` Change the stipple pixmap of face FACE to STIPPLE.
- · `set-face-underline` Specify whether face FACE is underlined.
- · `set-file-modes` Set mode bits of file named FILENAME to MODE (an integer).
- · `set-file-name-coding-system` Set coding system for decoding and encoding file names to CODING-SYSTEM.
- ✓ `set-fill-column` Set `fill-column' to specified argument.
- ✓ `set-fill-prefix` Set the fill prefix to the current line up to point.
- · `set-foreground-color` Set the foreground color of the selected frame to COLOR-NAME.
- · `set-frame-font` Set the default font to FONT.
- · `set-frame-height` Set text height of frame FRAME to HEIGHT lines.
- · `set-frame-name` Set the name of the selected frame to NAME.
- · `set-frame-width` Set text width of frame FRAME to WIDTH columns.
- · `set-fringe-style` Set the default appearance of fringes on the selected frame.
- ✓ `set-goal-column` Set the current horizontal position as a goal column.
- · `set-input-method` Select and activate input method INPUT-METHOD for the current buffer.
- · `set-justification` Set the region's justification style to STYLE.
- · `set-justification-center` Make paragraphs in the region centered.
- · `set-justification-full` Make paragraphs in the region fully justified.
- · `set-justification-left` Make paragraphs in the region left-justified.
- · `set-justification-none` Disable automatic filling for paragraphs in the region.
- · `set-justification-right` Make paragraphs in the region right-justified.
- · `set-keyboard-coding-system` Set coding system for keyboard input on TERMINAL to CODING-SYSTEM.
- · `set-language-environment` Set up multilingual environment for using LANGUAGE-NAME.
- · `set-left-margin` Set the left margin of the region to WIDTH.
- · `set-locale-environment` Set up multilingual environment for using LOCALE-NAME.
- ✓ `set-mark-command` Set the mark where point is, and activate it; or jump to the mark.
- · `set-mouse-color` Set the color of the mouse pointer of the selected frame to COLOR-NAME.
- · `set-next-selection-coding-system` Use CODING-SYSTEM for next communication with other window system clients.
- · `set-right-margin` Set the right margin of the region to WIDTH.
- · `set-rmail-inbox-list` Set the inbox list of the current RMAIL file to FILE-NAME.
- · `set-selection-coding-system` Make CODING-SYSTEM used for communicating with other X clients.
- · `set-selective-display` Set `selective-display' to ARG; clear it if no arg.
- · `set-terminal-coding-system` Set coding system of terminal output to CODING-SYSTEM.
- · `set-variable` Set VARIABLE to VALUE.  VALUE is a Lisp object.
- · `set-visited-file-name` Change name of file visited in current buffer to FILENAME.

## setenv

- · `setenv` Set the value of the environment variable named VARIABLE to VALUE.

## setup

- · `setup-specified-language-environment` Switch to a specified language environment.

## sgml

- · `sgml-mode` Major mode for editing SGML documents.

## sh

- · `sh-base-mode` Generic major mode for editing shell scripts.
- · `sh-mode` Major mode for editing shell scripts.

## shadow

- · `shadow-define-cluster` Edit (or create) the definition of a cluster NAME.
- · `shadow-define-literal-group` Declare a single file to be shared between sites.
- · `shadow-define-regexp-group` Make each of a group of files be shared between hosts.
- · `shadow-initialize` Set up file shadowing.

## shell

- · `shell` Run an inferior shell, with I/O through BUFFER (which defaults to `*shell*').
- ✓ `shell-command` Execute string COMMAND in inferior shell; display output, if any.
- · `shell-command-on-region` Execute string COMMAND in inferior shell with region as input.
- · `shell-script-mode` Major mode for editing shell scripts.

## shortdoc

- · `shortdoc` Pop to a buffer with short documentation summary for functions in GROUP.
- · `shortdoc-display-group` Pop to a buffer with short documentation summary for functions in GROUP.

## show

- · `show-paren-local-mode` Toggle `show-paren-mode' only in this buffer.
- · `show-paren-mode` Toggle visualization of matching parens (Show Paren mode).

## shr

- · `shr-render-region` Display the HTML rendering of the region between BEGIN and END.

## shrink

- ✓ `shrink-window` Make the selected window DELTA lines smaller.
- ✓ `shrink-window-horizontally` Make selected window DELTA columns narrower.
- · `shrink-window-if-larger-than-buffer` Shrink height of WINDOW if its buffer doesn't need so many lines.

## sieve

- · `sieve-manage` Open ManageSieve SERVER.
- · `sieve-mode` Major mode for editing Sieve code.
- · `sieve-upload` Upload script NAME to currently opened server.
- · `sieve-upload-and-bury` Upload script NAME and bury the current buffer.
- · `sieve-upload-and-kill` Upload script NAME and kill the current buffer.

## signal

- · `signal-process` Send PROCESS the signal with code SIGCODE.

## simula

- · `simula-mode` Major mode for editing SIMULA code.

## size

- · `size-indication-mode` Toggle buffer size display in the mode line (Size Indication mode).

## skeleton

- · `skeleton-pair-insert-maybe` Insert the character you type ARG times.

## slitex

- · `slitex-mode` Major mode for editing files of input for SliTeX.

## smerge

- · `smerge-ediff` Invoke ediff to resolve the conflicts.
- · `smerge-mode` Minor mode to simplify editing output from the diff3 program.
- · `smerge-start-session` Turn on `smerge-mode' and move point to first conflict marker.

## smiley

- · `smiley-buffer` Run `smiley-region' at the BUFFER, specified in the argument or
- · `smiley-region` Replace in the region `smiley-regexp-alist' matches with corresponding images.

## smtpmail

- · `smtpmail-send-queued-mail` Send mail that was queued as a result of setting `smtpmail-queue-mail'.

## snake

- · `snake` Play the Snake game.

## snmp

- · `snmp-mode` Major mode for editing SNMP MIBs.

## snmpv2

- · `snmpv2-mode` Major mode for editing SNMPv2 MIBs.

## so

- · `so-long` Invoke `so-long-action' and run `so-long-hook'.
- · `so-long-commentary` View the `so-long' library's documentation in `outline-mode'.
- · `so-long-customize` Open the customization group `so-long'.
- · `so-long-enable` Enable the `so-long' library's functionality.
- · `so-long-minor-mode` This is the minor mode equivalent of `so-long-mode'.
- · `so-long-mode` This major mode is the default `so-long-action' option.

## solitaire

- · `solitaire` Play Solitaire.

## sort

- · `sort-columns` Sort lines in region alphabetically by a certain range of columns.
- · `sort-fields` Sort lines in region lexicographically by the ARGth field of each line.
- ✓ `sort-lines` Sort lines in region alphabetically; REVERSE non-nil means descending order.
- · `sort-numeric-fields` Sort lines in region numerically by the ARGth field of each line.
- · `sort-pages` Sort pages in region alphabetically; argument means descending order.
- · `sort-paragraphs` Sort paragraphs in region alphabetically; argument means descending order.
- · `sort-regexp-fields` Sort the text in the region lexicographically.

## spam

- · `spam-initialize` Install the spam.el hooks and do other initialization.
- · `spam-report-agentize` Add spam-report support to the Agent.
- · `spam-report-deagentize` Remove spam-report support from the Agent.
- · `spam-report-process-queue` Report all queued requests from `spam-report-requests-file'.

## special

- · `special-mode` Parent major mode from which special major modes should inherit.

## speedbar

- · `speedbar` Open or close the `speedbar'.
- · `speedbar-frame` Enable or disable speedbar.
- · `speedbar-frame-mode` Enable or disable speedbar.
- · `speedbar-get-focus` Select speedbar window.
- · `speedbar-window` Enable or disable speedbar window mode.
- · `speedbar-window-mode` Enable or disable speedbar window mode.

## split

- · `split-frame` Split windows of specified FRAME into two separate frames.
- ✓ `split-line` Split current line, moving portion beyond point vertically down.
- ✓ `split-root-window-below` Split root window of current frame in two.
- ✓ `split-root-window-right` Split root window of current frame into two side-by-side windows.
- · `split-tab` Split windows of specified TAB into two separate tabs.
- ✓ `split-window-below` Split WINDOW-TO-SPLIT into two windows, one above the other.
- · `split-window-horizontally` Split WINDOW-TO-SPLIT into two side-by-side windows.
- ✓ `split-window-right` Split WINDOW-TO-SPLIT into two side-by-side windows.
- · `split-window-vertically` Split WINDOW-TO-SPLIT into two windows, one above the other.

## spook

- · `spook` Add that special touch of class to your outgoing mail.

## sql

- · `sql-connect` Connect to an interactive session using CONNECTION settings.
- · `sql-db2` Run db2 by IBM as an inferior process.
- · `sql-informix` Run dbaccess by Informix as an inferior process.
- · `sql-ingres` Run sql by Ingres as an inferior process.
- · `sql-interbase` Run isql by Interbase as an inferior process.
- · `sql-linter` Run inl by RELEX as an inferior process.
- · `sql-mariadb` Run mysql by MariaDB as an inferior process.
- · `sql-mode` Major mode to edit SQL.
- · `sql-ms` Run osql by Microsoft as an inferior process.
- · `sql-mysql` Run mysql by TcX as an inferior process.
- · `sql-oracle` Run sqlplus by Oracle as an inferior process.
- · `sql-postgres` Run psql by Postgres as an inferior process.
- · `sql-product-interactive` Run PRODUCT interpreter as an inferior process.
- · `sql-solid` Run solsql by Solid as an inferior process.
- · `sql-sqlite` Run sqlite as an inferior process.
- · `sql-sybase` Run isql by Sybase as an inferior process.
- · `sql-vertica` Run vsql as an inferior process.

## sqlite

- · `sqlite-mode-open-file` Browse the contents of an sqlite file.

## srecode

- · `srecode-template-mode` Major-mode for writing SRecode macros.

## srt

- · `srt-mode` Major-mode for writing SRecode macros.

## standard

- · `standard-display-by-replacement-char` Produce code to display characters between FROM and TO using REPL.
- · `standard-display-cyrillic-translit` Display a Cyrillic buffer using a transliteration.
- · `standard-display-unicode-special-glyphs` Display some glyphs using Unicode characters.

## start

- · `start-kbd-macro` Record subsequent keyboard input, defining a keyboard macro.

## string

- · `string-insert-rectangle` Insert STRING on each line of region-rectangle, shifting text right.
- ✓ `string-rectangle` Replace rectangle contents with STRING on each line.

## strokes

- · `strokes-compose-complex-stroke` Read a complex stroke and insert its glyph into the current buffer.
- · `strokes-decode-buffer` Decode stroke strings in BUFFER and display their corresponding glyphs.
- · `strokes-describe-stroke` Displays the command which STROKE maps to, reading STROKE interactively.
- · `strokes-do-complex-stroke` Read a complex stroke from the user and then execute its command.
- · `strokes-do-stroke` Read a simple stroke from the user and then execute its command.
- · `strokes-global-set-stroke` Interactively give STROKE the global binding as COMMAND.
- · `strokes-help` Get instruction on using the Strokes package.
- · `strokes-list-strokes` Pop up a buffer containing an alphabetical listing of strokes in STROKES-MAP.
- · `strokes-load-user-strokes` Load user-defined strokes from file named by `strokes-file'.
- · `strokes-mode` Toggle Strokes mode, a global minor mode.

## studlify

- · `studlify-buffer` Studlify-case the current buffer.
- · `studlify-region` Studlify-case the region.
- · `studlify-word` Studlify-case the current word, or COUNT words if given an argument.

## submit

- · `submit-emacs-patch` Send an Emacs patch to the Emacs maintainers.

## subword

- · `subword-mode` Toggle subword movement and editing (Subword mode).

## sunrise

- · `sunrise-sunset` Local time of sunrise and sunset for today.  Accurate to a few seconds.

## superword

- · `superword-mode` Toggle superword movement and editing (Superword mode).

## suspend

- · `suspend-emacs` Stop Emacs and return to superior process.  You can resume later.
- ✓ `suspend-frame` Do whatever is right to suspend the current frame.

## switch

- ✓ `switch-to-buffer` Display buffer BUFFER-OR-NAME in the selected window.
- · `switch-to-buffer-other-frame` Switch to buffer BUFFER-OR-NAME in another frame.
- · `switch-to-buffer-other-tab` Switch to buffer BUFFER-OR-NAME in another tab.
- ✓ `switch-to-buffer-other-window` Select the buffer specified by BUFFER-OR-NAME in another window.
- · `switch-to-completions` Select the completion list window.
- · `switch-to-minibuffer` Select the minibuffer window.
- · `switch-to-next-buffer` In WINDOW switch to next buffer.
- · `switch-to-prev-buffer` In WINDOW switch to previous buffer.

## system

- · `system-taskbar-mode` System GUI taskbar icon badge, progress report, alerting.

## tab

- · `tab-bar-change-tab-group` Add the tab specified by its absolute position TAB-NUMBER to GROUP-NAME.
- · `tab-bar-close-group-tabs` Close all tabs that belong to GROUP-NAME on the selected frame.
- · `tab-bar-close-other-tabs` Close all tabs on the selected frame, except the tab TAB-NUMBER.
- · `tab-bar-close-tab` Close the tab specified by its absolute position TAB-NUMBER.
- · `tab-bar-close-tab-by-name` Close the tab given its NAME.
- · `tab-bar-detach-tab` Move tab number FROM-NUMBER to a new frame.
- · `tab-bar-duplicate-tab` Clone the current tab to ARG positions to the right.
- · `tab-bar-history-back` Restore a previous window configuration used in the current tab.
- · `tab-bar-history-forward` Cancel restoration of the previous window configuration.
- · `tab-bar-history-mode` Toggle tab history mode for the tab bar.
- · `tab-bar-menu-bar` Pop up the same menu as displayed by the menu bar.
- · `tab-bar-merge-tabs` Merge the main window of TAB2 into TAB1.
- · `tab-bar-mode` Toggle the tab bar in all graphical frames (Tab Bar mode).
- · `tab-bar-mouse-1` Close the tab whose "x" close button you click.
- · `tab-bar-mouse-close-tab` Close the tab you click on.
- · `tab-bar-mouse-context-menu` Pop up the context menu for the tab on which you click.
- · `tab-bar-mouse-down-1` Select the tab at mouse click, or add a new tab on the tab bar.
- · `tab-bar-mouse-move-tab` Move a tab to a different position on the tab bar.
- · `tab-bar-move-tab` Move the current tab ARG positions to the right.
- · `tab-bar-move-tab-backward` Move the current tab ARG positions to the left.
- · `tab-bar-move-tab-to` Move tab from FROM-NUMBER position to new position at TO-NUMBER.
- · `tab-bar-move-tab-to-frame` Move tab from FROM-NUMBER position to new position at TO-NUMBER.
- · `tab-bar-move-tab-to-group` Relocate TAB (by default, the current tab) closer to its group.
- · `tab-bar-move-window-to-tab` Move the selected window to a new tab.
- · `tab-bar-new-tab` Create a new tab ARG positions to the right.
- · `tab-bar-new-tab-to` Add a new tab at the absolute position TAB-NUMBER.
- · `tab-bar-rename-tab` Give the tab specified by its absolute position TAB-NUMBER a new NAME.
- · `tab-bar-rename-tab-by-name` Rename the tab named TAB-NAME to NEW-NAME.
- · `tab-bar-select-tab` Switch to the tab by its absolute position TAB-NUMBER in the tab bar.
- · `tab-bar-select-tab-by-name` Switch to the tab by NAME.
- · `tab-bar-split-tab` Split windows of specified TAB into two separate tabs.
- · `tab-bar-switch-to-last-tab` Switch to the last tab or ARGth tab from the end of the tab bar.
- · `tab-bar-switch-to-next-tab` Switch to ARGth next tab.
- · `tab-bar-switch-to-prev-tab` Switch to ARGth previous tab.
- · `tab-bar-switch-to-recent-tab` Switch to ARGth most recently visited tab.
- · `tab-bar-switch-to-tab` Switch to the tab by NAME.
- · `tab-bar-touchscreen-begin` Handle a touchscreen begin EVENT on the tab bar.
- · `tab-bar-undo-close-tab` Restore the most recently closed tab.
- · `tab-close` Close the tab specified by its absolute position TAB-NUMBER.
- · `tab-close-group` Close all tabs that belong to GROUP-NAME on the selected frame.
- · `tab-close-other` Close all tabs on the selected frame, except the tab TAB-NUMBER.
- · `tab-detach` Move tab number FROM-NUMBER to a new frame.
- · `tab-duplicate` Clone the current tab to ARG positions to the right.
- · `tab-group` Add the tab specified by its absolute position TAB-NUMBER to GROUP-NAME.
- · `tab-last` Switch to the last tab or ARGth tab from the end of the tab bar.
- · `tab-line-mode` Toggle display of tab line in the windows displaying the current buffer.
- · `tab-list` Display a list of named window configurations.
- · `tab-move` Move the current tab ARG positions to the right.
- · `tab-move-to` Move tab from FROM-NUMBER position to new position at TO-NUMBER.
- · `tab-new` Create a new tab ARG positions to the right.
- · `tab-new-to` Add a new tab at the absolute position TAB-NUMBER.
- · `tab-next` Switch to ARGth next tab.
- · `tab-previous` Switch to ARGth previous tab.
- · `tab-recent` Switch to ARGth most recently visited tab.
- · `tab-rename` Give the tab specified by its absolute position TAB-NUMBER a new NAME.
- · `tab-select` Switch to the tab by its absolute position TAB-NUMBER in the tab bar.
- · `tab-switch` Switch to the tab by NAME.
- · `tab-switcher` Display a list of named window configurations.
- · `tab-switcher-backup-unmark` Move up one line and cancel operations on window configuration there.
- · `tab-switcher-delete` Mark window configuration on this line to be deleted.
- · `tab-switcher-delete-backwards` Mark window configuration on this line to be deleted.
- · `tab-switcher-execute` Delete the marked window configurations.
- · `tab-switcher-mode` Major mode for selecting a window configuration.
- · `tab-switcher-mouse-select` Select the window configuration whose line you click on.
- · `tab-switcher-next-line` Move to ARGth next line in the list of tabs.
- · `tab-switcher-prev-line` Move to ARGth previous line in the list of tabs.
- · `tab-switcher-select` Select this line's window configuration.
- · `tab-switcher-unmark` Cancel operations on window configuration on this line and move down.
- ✓ `tab-to-tab-stop` Insert spaces or tabs to next defined tab-stop column.
- · `tab-undo` Restore the most recently closed tab.
- · `tab-window-detach` Move the selected window to a new tab.

## tabify

- · `tabify` Convert multiple spaces in region to tabs when possible.

## table

- · `table-backward-cell` Move backward to the beginning of the previous cell.
- · `table-capture` Convert plain text into a table by capturing the text in the region.
- · `table-delete-column` Delete N column(s) of cells.
- · `table-delete-row` Delete N row(s) of cells.
- · `table-fixed-width-mode` Cell width is fixed when this is non-nil.
- · `table-forward-cell` Move point forward to the beginning of the next cell.
- · `table-generate-source` Generate source of the current table in the specified language.
- · `table-heighten-cell` Heighten the current cell by N lines by expanding the cell vertically.
- · `table-insert` Insert an editable text table.
- · `table-insert-column` Insert N table column(s).
- · `table-insert-row` Insert N table row(s).
- · `table-insert-row-column` Insert row(s) or column(s).
- · `table-insert-sequence` Travel cells forward while inserting a specified sequence string in each cell.
- · `table-justify` Justify contents of a cell, a row of cells or a column of cells.
- · `table-justify-cell` Justify cell contents.
- · `table-justify-column` Justify cells of a column.
- · `table-justify-row` Justify cells of a row.
- · `table-narrow-cell` Narrow the current cell by N columns and shrink the cell horizontally.
- · `table-query-dimension` Return the dimension of the current cell and the current table.
- · `table-recognize` Recognize all tables within the current buffer and activate them.
- · `table-recognize-cell` Recognize a table cell that contains current point.
- · `table-recognize-region` Recognize all tables within region.
- · `table-recognize-table` Recognize a table at point.
- · `table-release` Convert a table into plain text by removing the frame from a table.
- · `table-shorten-cell` Shorten the current cell by N lines by shrinking the cell vertically.
- · `table-span-cell` Span current cell into adjacent cell in DIRECTION.
- · `table-split-cell` Split current cell in ORIENTATION.
- · `table-split-cell-horizontally` Split current cell horizontally.
- · `table-split-cell-vertically` Split current cell vertically.
- · `table-unrecognize` 
- · `table-unrecognize-cell` 
- · `table-unrecognize-region` 
- · `table-unrecognize-table` 
- · `table-widen-cell` Widen the current cell by N columns and expand the cell horizontally.

## tabulated

- · `tabulated-list-col-sort` Sort Tabulated List entries by the column of the mouse click E.
- · `tabulated-list-mode` Generic major mode for browsing a list of items.
- · `tabulated-list-narrow-current-column` Narrow the current tabulated list column by N chars.
- · `tabulated-list-next-column` Go to the start of the next column after point on the current line.
- · `tabulated-list-previous-column` Go to the start of the column point is in on the current line.
- · `tabulated-list-revert` The `revert-buffer-function' for `tabulated-list-mode'.
- · `tabulated-list-sort` Sort Tabulated List entries by the column at point.
- · `tabulated-list-widen-current-column` Widen the current tabulated-list column by N chars.

## tags

- · `tags-apropos` Display list of all tags in tags table REGEXP matches.
- · `tags-loop-continue` Continue last \[tags-search] or \[tags-query-replace] command.
- · `tags-next-file` Select next file among files in current tags table.
- · `tags-query-replace` Do `query-replace-regexp' of FROM with TO on all files listed in tags table.
- · `tags-reset-tags-tables` Reset tags state to cancel effect of any previous \[visit-tags-table] or \[find-tag].
- · `tags-search` Search through all files listed in tags table for match for REGEXP.
- · `tags-table-mode` Major mode for tags table file buffers.

## talk

- · `talk` Connect to the Emacs talk group from the current X display or tty frame.
- · `talk-connect` Connect to display DISPLAY for the Emacs talk group.

## tar

- · `tar-mode` Major mode for viewing a tar file as a dired-like listing of its contents.

## tcl

- · `tcl-help-on-word` Get help on Tcl command.  Default is word at point.
- · `tcl-mode` Major mode for editing Tcl code.

## tear

- · `tear-off-window` Delete the selected window, and create a new frame displaying its buffer.

## telnet

- · `telnet` Open a network login connection to host named HOST (a string).

## temp

- · `temp-buffer-resize-mode` Toggle auto-resizing temporary buffer windows (Temp Buffer Resize Mode).

## term

- · `term` Start a terminal-emulator in a new buffer.

## testcover

- · `testcover-start` Use Edebug to instrument for coverage all macros and functions in FILENAME.
- · `testcover-this-defun` Start coverage on function under point.

## tetris

- · `tetris` Play the Tetris game.

## tex

- · `tex-mode` Major mode for editing files of input for TeX, LaTeX, or SliTeX.

## TeX

- · `TeX-mode` Major mode for editing files of input for TeX, LaTeX, or SliTeX.

## texi2info

- · `texi2info` Convert the current buffer (written in Texinfo code) into an Info file.

## texinfo

- · `texinfo-format-buffer` Process the current buffer as texinfo code, into an Info file.
- · `texinfo-format-region` Convert the current region of the Texinfo file to Info format.
- · `texinfo-mode` Major mode for editing Texinfo files.

## text

- · `text-mode` Major mode for editing text written for humans to read.
- · `text-mode-menu` Menu for `text-mode'.
- · `text-scale-adjust` Adjust the font size in the current buffer by INC steps.
- · `text-scale-decrease` Decrease the font size of the default face in the current buffer by DEC steps.
- · `text-scale-increase` Increase the font size of the default face in current buffer by INC steps.
- · `text-scale-pinch` Adjust the height of the default face by the scale in the pinch event EVENT.
- · `text-scale-set` Set the scale factor of the default face in the current buffer to LEVEL.

## thai

- · `thai-compose-buffer` Compose Thai characters in the current buffer.
- · `thai-compose-region` Compose Thai characters in the region.

## theme

- · `theme-choose-variant` Switch from the current theme to one of its variants.

## thread

- · `thread-handle-event` Handle thread events, propagated by `thread-signal'.

## tibetan

- · `tibetan-compose-buffer` Composes Tibetan character components in the buffer.
- · `tibetan-compose-region` Compose Tibetan text the region BEG and END.
- · `tibetan-decompose-buffer` Decomposes Tibetan characters in the buffer into their components.
- · `tibetan-decompose-region` Decompose Tibetan text in the region FROM and TO.

## tildify

- · `tildify-buffer` Add hard spaces in the current buffer.
- · `tildify-mode` Adds electric behavior to space character.
- · `tildify-region` Add hard spaces in the region between BEG and END.
- · `tildify-space` Convert space before point into a hard space if the context is right.

## time

- · `time-stamp` Update any time stamp strings (timestamps) in the buffer.
- · `time-stamp-toggle-active` Set `time-stamp-active' (whether \[time-stamp] updates a buffer).

## timeclock

- · `timeclock-change` Change to working on a different project.
- · `timeclock-in` Clock in, recording the current time moment in the timelog.
- · `timeclock-mode-line-display` Toggle display of the amount of time left today in the mode line.
- · `timeclock-out` Clock out, recording the current time moment in the timelog.
- · `timeclock-reread-log` Re-read the timeclock, to account for external changes.
- · `timeclock-status-string` Report the overall timeclock status at the present moment.
- · `timeclock-when-to-leave-string` Return a string representing the end of today's workday.
- · `timeclock-workday-elapsed-string` Return a string representing the amount of time worked today.
- · `timeclock-workday-remaining-string` Return a string representing the amount of time left today.

## tmm

- · `tmm-menubar` Text-mode emulation of looking and choosing from a menubar.
- · `tmm-menubar-mouse` Text-mode emulation of looking and choosing from a menubar.

## todo

- · `todo-archive-mode` Major mode for archived todo categories.
- · `todo-filtered-items-mode` Mode for displaying and reprioritizing top priority Todo.
- · `todo-mode` Major mode for displaying, navigating and editing todo lists.
- · `todo-show` Visit a todo file and display one of its categories.

## toggle

- · `toggle-auto-composition` Toggle Auto Composition mode.
- · `toggle-case-fold-search` Toggle whether to ignore letter-case in search commands.
- · `toggle-debug-on-error` Toggle whether to enter Lisp debugger when an error is signaled.
- · `toggle-debug-on-quit` Toggle whether to enter Lisp debugger when C-g is pressed.
- · `toggle-enable-multibyte-characters` Change whether this buffer uses multibyte characters.
- · `toggle-frame-fullscreen` Toggle fullscreen state of FRAME.
- · `toggle-frame-maximized` Toggle maximization state of FRAME.
- · `toggle-frame-tab-bar` Toggle tab bar of the selected frame.
- · `toggle-horizontal-scroll-bar` Toggle whether or not the selected frame has horizontal scroll bars.
- · `toggle-indicate-empty-lines` Toggle whether to indicate trailing empty lines in fringe, globally.
- · `toggle-input-method` Enable or disable multilingual text input method for the current buffer.
- · `toggle-menu-bar-mode-from-frame` Toggle display of the menu bar.
- · `toggle-option` Toggle the value of boolean option SYMBOL for this session.
- · `toggle-rot13-mode` Toggle the use of ROT13 encoding for the current window.
- · `toggle-save-desktop-globally` Toggle whether to visit desktop of previous session when restarting Emacs.
- · `toggle-save-place-globally` Toggle whether to visit files of previous session when restarting Emacs.
- · `toggle-scroll-bar` Toggle whether or not the selected frame has vertical scroll bars.
- · `toggle-tab-bar-mode-from-frame` Toggle tab bar on or off, based on the status of the current frame.
- · `toggle-text-mode-auto-fill` Toggle whether to use Auto Fill in Text mode and related modes.
- · `toggle-theme` Switch from the current theme to one of its variants.
- · `toggle-tool-bar-mode-from-frame` Toggle tool bar on or off, based on the status of the current frame.
- ✓ `toggle-truncate-lines` Toggle truncating of long lines for the current buffer.
- · `toggle-uniquify-buffer-names` Toggle whether to uniquify buffer names by adding parent directory names.
- · `toggle-use-system-font` Toggle whether to use the monospaced font defined by the system.
- · `toggle-viper-mode` Toggle Viper on/off.
- · `toggle-window-dedicated` Toggle whether WINDOW is dedicated to its current buffer.
- · `toggle-word-wrap` Toggle whether to use word-wrapping for continuation lines.

## toml

- · `toml-ts-mode` Major mode for editing TOML, powered by tree-sitter.

## tool

- · `tool-bar-mode` Toggle the tool bar in all graphical frames (Tool Bar mode).

## tooltip

- · `tooltip-mode` Toggle Tooltip mode.

## top

- · `top-level` Exit all recursive editing levels.

## touch

- · `touch-screen-drag` Handle a drag EVENT by setting the region to its new point.
- · `touch-screen-handle-touch` Handle a single touch EVENT, and perform associated actions.
- · `touch-screen-hold` Handle a long press EVENT.
- · `touch-screen-pinch` Scroll the window in the touchscreen-pinch event EVENT.
- · `touch-screen-preview-select` Display a preview of the line around point in the echo area.
- · `touch-screen-restart-drag` Restart dragging to select text.
- · `touch-screen-scroll` Scroll the window within EVENT, a `touchscreen-scroll' event.

## trace

- · `trace-function` Trace calls to function FUNCTION.
- · `trace-function-background` Trace calls to function FUNCTION, quietly.
- · `trace-function-foreground` Trace calls to function FUNCTION.

## traceroute

- · `traceroute` Run `traceroute-program' for TARGET.

## tramp

- · `tramp-dired-find-file-with-sudo` Visit the file or directory named on this line as the superuser.
- · `tramp-revert-buffer-with-sudo` Visit the current file again with superuser, or root, permissions.
- · `tramp-unload-tramp` Discard Tramp from loading remote files.

## transient

- · `transient-mark-mode` Toggle Transient Mark mode.

## translate

- · `translate-region` From START to END, translate characters according to TABLE.

## transpose

- ✓ `transpose-chars` Interchange characters around point, moving forward one character.
- ✓ `transpose-lines` Exchange current line and previous line, leaving point after both.
- · `transpose-paragraphs` Interchange the current paragraph with the next one.
- · `transpose-regions` Transpose region STARTR1 to ENDR1 with STARTR2 to ENDR2.
- · `transpose-sentences` Interchange the current sentence with the next one.
- ✓ `transpose-sexps` Like \[transpose-chars] (`transpose-chars'), but applies to sexps.
- ✓ `transpose-words` Interchange words around point, leaving point at end of them.

## treesit

- · `treesit-install-language-grammar` Build and install the tree-sitter language grammar library for LANG.

## tsx

- · `tsx-ts-mode` Major mode for editing TSX and JSX documents.

## tty

- · `tty-tip-mode` Global minor mode for displaying help in tty child frames.

## tumme

- · `tumme` Make a thumbnail buffer for all images in DIRNAME and display it.

## turn

- · `turn-on-flyspell` Toggle on-the-fly spell checking (Flyspell mode).
- · `turn-on-gnus-dired-mode` Convenience method to turn on `gnus-dired-mode'.

## type

- · `type-break` Take a typing break.
- · `type-break-guesstimate-keystroke-threshold` Guess values for the minimum/maximum keystroke threshold for typing breaks.
- · `type-break-mode` Enable or disable typing-break mode.
- · `type-break-statistics` Print statistics about typing breaks in a temporary buffer.

## typescript

- · `typescript-ts-base-mode` Generic major mode for editing TypeScript.
- · `typescript-ts-mode` Major mode for editing TypeScript.

## unbury

- · `unbury-buffer` Switch to the last buffer in the buffer list.

## uncomment

- · `uncomment-region` Uncomment each line in the BEG .. END region.

## undefined

- · `undefined` Beep to tell the user this binding is undefined.

## undelete

- · `undelete-frame` Undelete a frame deleted with `delete-frame'.
- · `undelete-frame-by-id` Undelete the frame with the matching ID.
- · `undelete-frame-mode` Enable the `undelete-frame' command.

## underline

- · `underline-region` Underline all nonblank characters in the region.

## undigestify

- · `undigestify-rmail-message` Break up a digest message into its constituent messages.

## undo

- ✓ `undo` Undo some previous changes.
- · `undo-ignore-read-only` Perform `undo', ignoring the buffer's read-only status.
- ✓ `undo-only` Undo some previous changes.
- ✓ `undo-redo` Undo the last ARG undos, i.e., redo the last ARG changes.

## unexpand

- · `unexpand-abbrev` Undo the expansion of the last abbrev that expanded.

## unfill

- · `unfill-paragraph` Join lines of this paragraph and fix up whitespace at joins.

## unforward

- · `unforward-rmail-message` Extract a forwarded message from the containing message.

## unhighlight

- ✓ `unhighlight-regexp` Remove highlighting of each match to REGEXP set by hi-lock.

## uniquify

- · `uniquify-rationalize-file-buffer-names` Make file buffer names unique by adding segments from file name.

## universal

- ✓ `universal-argument` Begin a numeric argument for the following command.
- · `universal-argument-more` 
- · `universal-coding-system-argument` Execute an I/O command using the specified CODING-SYSTEM.

## unix

- · `unix-filename-rubout` Kill ARG Unix-words backwards, also treating slashes as word delimiters.
- · `unix-sync` Tell Unix to finish all pending disk updates.
- · `unix-word-rubout` Kill ARG Unix-words backwards.

## unload

- · `unload-feature` Unload the library that provided FEATURE.

## unmorse

- · `unmorse-region` Convert Morse coded text in region to plain text.

## unrmail

- · `unrmail` Convert old-style Rmail Babyl file FILE to mbox format file TO-FILE.

## untabify

- · `untabify` Convert all tabs in region to multiple spaces, preserving columns.

## ununderline

- · `ununderline-region` Remove all underlining (overstruck underscores) in the region.

## up

- ✓ `up-list` Move forward out of one level of parentheses.

## upcase

- · `upcase-char` Uppercasify ARG chars starting from point.  Point doesn't move.
- · `upcase-dwim` Upcase words in the region, if active; if not, upcase word at point.
- · `upcase-initials-region` Upcase the initial of each word in the region.
- ✓ `upcase-region` Convert the region to upper case.  In programs, wants two arguments.
- ✓ `upcase-word` Convert to upper case from point to end of word, moving over.

## update

- · `update-directory-autoloads` 

## url

- · `url-gateway-nslookup-host` Attempt to resolve the given HOST using nslookup if possible.
- · `url-handler-mode` Handle URLs as if they were file names throughout Emacs.
- · `url-mail` Start editing a mail message to be sent.
- · `url-setup-privacy-info` Setup variables that expose info about you and your system.
- · `url-view-url` View the current document's URL.

## use

- · `use-hard-newlines` Toggle between hard and soft newlines in the current buffer.
- · `use-package-jump-to-package-form` Attempt to find and jump to the `use-package' form that loaded PACKAGE.
- · `use-package-lint` Check for errors in `use-package' declarations.

## uudecode

- · `uudecode-decode-region-external` Uudecode region between START and END using external program.
- · `uudecode-decode-region-internal` Uudecode region between START and END without using an external program.

## variable

- · `variable-pitch-mode` Variable-pitch default-face mode.

## vc

- ✓ `vc-add-working-tree` Create working tree DIRECTORY with same backing repository as this tree.
- ✓ `vc-annotate` Display the edit history of the current FILE using colors.
- · `vc-apply-root-to-other-working-tree` Apply all local changes in this working tree to the tree under DIRECTORY.
- · `vc-apply-to-other-working-tree` Apply VC fileset's local changes to working tree under DIRECTORY.
- · `vc-auto-revert-mode` Toggle Auto-Revert mode in many buffers.
- · `vc-change-backend` Make BACKEND the current version control system for FILE.
- · `vc-cherry-pick` Copy the changes from a single revision REV to the current branch.
- ✓ `vc-create-branch` Make a branch called NAME in directory DIR.
- ✓ `vc-create-tag` Descending recursively from DIR, make a tag called NAME.
- ✓ `vc-delete-file` Delete file and mark it as such in the version control system.
- · `vc-delete-revision` Delete revision REV from the revision history.
- · `vc-delete-revisions-from-end` Delete revisions newer than REV.
- ✓ `vc-delete-working-tree` Delete working tree DIRECTORY with same backing repository as this tree.
- ✓ `vc-diff` Display diffs between file revisions.
- · `vc-diff-incoming` Report changes to VC fileset that would be pulled from UPSTREAM-LOCATION.
- ✓ `vc-diff-mergebase` Report diffs between the merge base of REV1 and REV2 revisions.
- · `vc-diff-outgoing` Report changes to VC fileset that would be pushed to UPSTREAM-LOCATION.
- ✓ `vc-diff-outgoing-and-edited` Report combined diff of outgoing and uncommitted changes to VC fileset.
- ✓ `vc-diff-remote-unintegrated` Show remote fileset changes since merge base with UPSTREAM-LOCATION.
- ✓ `vc-diff-unintegrated` Report changes to VC fileset since the merge base with UPSTREAM-LOCATION.
- ✓ `vc-dir` Show the VC status for "interesting" files in and below DIR.
- · `vc-dir-root` Run `vc-dir' in the repository root directory without prompt.
- · `vc-ediff` Display diffs between file revisions using ediff.
- · `vc-edit-next-command` Request editing the next VC shell command before execution.
- · `vc-find-conflicted-file` Visit the next conflicted file in the current project.
- · `vc-find-file-hook` Refresh the VC state of the current buffer's file.
- · `vc-git-grep` Run git grep, searching for REGEXP in FILES in directory DIR.
- ✓ `vc-ignore` Ignore FILE under the VCS of DIRECTORY.
- · `vc-insert-headers` Insert headers into a file for use with a version control system.
- · `vc-kill-other-working-tree-buffers` Kill buffers visiting versions of this file in other working trees.
- ✓ `vc-log-mergebase` Show a log of changes between the merge base of revisions REV1 and REV2.
- ✓ `vc-log-remote-unintegrated` Show remote log for VC fileset since merge base with UPSTREAM-LOCATION.
- · `vc-log-search` Search the VC log of changes for PATTERN and show log of matching changes.
- ✓ `vc-log-unintegrated` Show log for the VC fileset since the merge base with UPSTREAM-LOCATION.
- ✓ `vc-merge` Perform a version control merge operation.
- · `vc-mode-line` Set `vc-mode' to display type of version control for FILE.
- ✓ `vc-move-working-tree` Relocate a working tree from FROM to TO, two directory file names.
- ✓ `vc-next-action` Do the next logical version control operation on the current fileset.
- · `vc-prepare-patch` Compose an Email sending patches for REVISIONS to ADDRESSEE.
- · `vc-print-change-log` Show in another window the VC change history of the current fileset.
- ✓ `vc-print-fileset-branch-log` Show log of VC changes on BRANCH, limited to the current fileset.
- ✓ `vc-print-log` Show in another window the VC change history of the current fileset.
- ✓ `vc-print-root-branch-log` Show root log of VC changes on BRANCH in another window.
- · `vc-print-root-change-log` Show in another window the VC change history of the whole tree.
- ✓ `vc-print-root-log` Show in another window VC change history of the current VC controlled tree.
- · `vc-pull` Update the current fileset or branch.
- · `vc-pull-and-push` First pull, and then push the current branch.
- ✓ `vc-push` Push the current branch.
- · `vc-refresh-state` Refresh the VC state of the current buffer's file.
- ✓ `vc-region-history` Show the history of the region between FROM and TO.
- ✓ `vc-register` Register into a version control system.
- ✓ `vc-rename-file` Rename file OLD to NEW in both working tree and repository.
- · `vc-resolve-conflicts` Invoke ediff to resolve the conflicts.
- · `vc-restore` Revert working copies of the selected fileset to their repository contents.
- ✓ `vc-retrieve-tag` For each file in or below DIR, retrieve their version identified by tag NAME.
- ✓ `vc-revert` Revert working copies of the selected fileset to their repository contents.
- · `vc-revert-or-delete-revision` Undo the effects of revision REV.
- · `vc-revert-revision` Make a commit undoing the effects of revision REV.
- ✓ `vc-revision-other-window` Visit revision REV of the current file in another window.
- ✓ `vc-root-diff` Display diffs between VC-controlled whole tree revisions.
- · `vc-root-diff-incoming` Report diff of all changes that would be pulled from UPSTREAM-LOCATION.
- · `vc-root-diff-outgoing` Report diff of all changes that would be pushed to UPSTREAM-LOCATION.
- ✓ `vc-root-diff-outgoing-and-edited` Report combined diff of all outgoing and uncommitted changes.
- ✓ `vc-root-diff-remote-unintegrated` Report diff of remote changes since merge base with UPSTREAM-LOCATION.
- ✓ `vc-root-diff-unintegrated` Report diff of all changes since the merge base with UPSTREAM-LOCATION.
- ✓ `vc-root-log-incoming` Show log of changes that will be received with pull from UPSTREAM-LOCATION.
- ✓ `vc-root-log-outgoing` Show log of changes that will be sent with a push to UPSTREAM-LOCATION.
- ✓ `vc-root-log-remote-unintegrated` Show log of remote revisions since merge base with UPSTREAM-LOCATION.
- ✓ `vc-root-log-unintegrated` Show log of revisions since the merge base with UPSTREAM-LOCATION.
- · `vc-root-version-diff` Report diffs between REV1 and REV2 revisions of the whole tree.
- ✓ `vc-switch-branch` Switch to the branch NAME in the directory DIR.
- ✓ `vc-switch-working-tree` Switch to the version of this file in working tree under DIRECTORY.
- · `vc-uncommit-revisions-from-end` Delete revisions newer than REV without touching the working tree.
- ✓ `vc-update` Update the current fileset or branch.
- · `vc-update-change-log` Find change log file and add entries from recent version control logs.
- · `vc-version-diff` Report diffs between revisions REV1 and REV2 in the repository history.
- · `vc-version-ediff` Show differences between REV1 and REV2 of FILES using ediff.
- ✓ `vc-working-tree-switch-project` Like \[project-switch-project] but limited to projects with the same backing repository.

## vera

- · `vera-mode` Major mode for editing Vera code.

## verilog

- · `verilog-mode` Major mode for editing Verilog code.

## version

- · `version` Display the version of Emacs that is running in this session.

## vhdl

- · `vhdl-mode` Major mode for editing VHDL code.

## viet

- · `viet-decode-viqr-buffer` Convert `VIQR' mnemonics of the current buffer to Vietnamese characters.
- · `viet-decode-viqr-region` Convert `VIQR' mnemonics of the current region to Vietnamese characters.
- · `viet-encode-viqr-buffer` Convert Vietnamese characters of the current buffer to `VIQR' mnemonics.
- · `viet-encode-viqr-region` Convert Vietnamese characters of the current region to `VIQR' mnemonics.

## view

- · `view-buffer` View BUFFER in View mode, returning to previous buffer when done.
- · `view-buffer-other-frame` View BUFFER in View mode in another frame.
- · `view-buffer-other-window` View BUFFER in View mode in another window.
- ✓ `view-echo-area-messages` View the log of recent echo-area messages: the `*Messages*' buffer.
- · `view-emacs-debugging` Display info on how to debug Emacs problems.
- · `view-emacs-FAQ` Display the Emacs Frequently Asked Questions (FAQ) file.
- · `view-emacs-news` Display info on recent changes to Emacs.
- · `view-emacs-problems` Display info on known problems with Emacs and possible workarounds.
- · `view-emacs-todo` Display the Emacs TODO list.
- · `view-external-packages` Display info on where to get more Emacs packages.
- · `view-file` View FILE in View mode, returning to previous buffer when done.
- · `view-file-other-frame` View FILE in View mode in another frame.
- · `view-file-other-window` View FILE in View mode in another window.
- · `view-hello-file` Display the HELLO file, which lists many languages and characters.
- ✓ `view-lossage` Display last few input keystrokes and the commands run.
- · `view-mode` Toggle View mode, a minor mode for viewing text but not editing it.
- · `view-order-manuals` Display information on how to buy printed copies of Emacs manuals.
- · `view-register` Display the description of the contents of REGISTER.

## View

- · `View-exit-and-edit` Exit View mode and make the current buffer editable.

## viper

- · `viper-mode` Turn on Viper emulation of Vi in Emacs.  See Info node `(viper)Top'.

## visible

- · `visible-mode` Toggle making all invisible text temporarily visible (Visible mode).

## visit

- · `visit-tags-table` Tell tags commands to use tags table file FILE.

## visual

- · `visual-line-mode` Toggle visual line based editing (Visual Line mode) in the current buffer.
- · `visual-wrap-prefix-mode` Display continuation lines with prefixes from surrounding context.

## wallpaper

- · `wallpaper-set` Set the desktop background to FILE in a graphical environment.

## wdired

- · `wdired-change-to-wdired-mode` Put a Dired buffer in Writable Dired (WDired) mode.

## webjump

- · `webjump` Jumps to a Web site from a programmable hotlist.

## what

- ✓ `what-cursor-position` Print info on cursor position (on screen and within buffer).
- · `what-domain` Convert mail domain DOMAIN to the country it corresponds to.
- · `what-line` Print the current buffer line number and narrowed line number of point.
- · `what-page` Display the page number, and the line number within that page.

## where

- ✓ `where-is` Print message listing key sequences that invoke the command DEFINITION.

## which

- · `which-function-mode` Toggle mode line display of current function (Which Function mode).
- · `which-key-C-h-dispatch` Dispatch \`C-h' commands by looking up key in `which-key-C-h-map'.
- · `which-key-dump-bindings` Dump bindings from PREFIX into buffer named BUFFER-NAME.
- · `which-key-mode` Toggle `which-key-mode'.
- · `which-key-setup-minibuffer` Set up minibuffer display.
- · `which-key-setup-side-window-bottom` Set up side-window that opens on bottom.
- · `which-key-setup-side-window-right` Set up side-window on right.
- · `which-key-setup-side-window-right-bottom` Set up side-window on right if space allows.
- · `which-key-show-full-keymap` Show all bindings in KEYMAP using which-key.
- · `which-key-show-full-major-mode` Show all bindings in the map of the current major mode.
- · `which-key-show-full-minor-mode-keymap` Show all bindings in KEYMAP using which-key.
- · `which-key-show-keymap` Show the top-level bindings in KEYMAP using which-key.
- · `which-key-show-major-mode` Show top-level bindings in the map of the current major mode.
- · `which-key-show-minor-mode-keymap` Show the top-level bindings in KEYMAP using which-key.
- · `which-key-show-next-page-cycle` Show the next page of keys, cycling from end to beginning.
- · `which-key-show-next-page-no-cycle` Show next page of keys or `which-key-show-standard-help'.
- · `which-key-show-previous-page-cycle` Show the previous page of keys, cycling from beginning to end.
- · `which-key-show-previous-page-no-cycle` Show previous page of keys if one exists.
- · `which-key-show-standard-help` Call the command in `which-key--prefix-help-cmd-backup'.
- · `which-key-show-top-level` Show top-level bindings.
- · `which-key-undo-key` Undo last keypress and force which-key update.

## whitespace

- · `whitespace-cleanup` Cleanup some blank problems in all buffer or at region.
- · `whitespace-cleanup-region` Cleanup some blank problems at region.
- · `whitespace-mode` Toggle whitespace visualization (Whitespace mode).
- · `whitespace-newline-mode` Toggle newline visualization (Whitespace Newline mode).
- · `whitespace-page-delimiters-mode` Display page-break delimiter characters as horizontal lines.
- · `whitespace-report` Report some whitespace problems in buffer.
- · `whitespace-report-region` Report some whitespace problems in a region.
- · `whitespace-toggle-options` Toggle local `whitespace-mode' options.

## whois

- · `whois` Send SEARCH-STRING to server defined by the `whois-server-name' variable.
- · `whois-reverse-lookup` 

## widen

- ✓ `widen` Remove restrictions (narrowing) from current buffer.

## widget

- · `widget-browse` Create a widget browser for WIDGET.
- · `widget-browse-at` Browse the widget under point.
- · `widget-browse-other-window` Show widget browser for WIDGET in other window.
- · `widget-minor-mode` Minor mode for traversing widgets.

## windmove

- · `windmove-default-keybindings` Set up keybindings for `windmove'.
- · `windmove-delete-default-keybindings` Set up keybindings for directional window deletion.
- · `windmove-delete-down` Delete the window below the current one.
- · `windmove-delete-left` Delete the window to the left of the current one.
- · `windmove-delete-right` Delete the window to the right of the current one.
- · `windmove-delete-up` Delete the window above the current one.
- · `windmove-display-default-keybindings` Set up keybindings for directional buffer display.
- · `windmove-display-down` Display the next buffer in window below the current one.
- · `windmove-display-left` Display the next buffer in window to the left of the current one.
- · `windmove-display-new-frame` Display the next buffer in a new frame.
- · `windmove-display-new-tab` Display the next buffer in a new tab.
- · `windmove-display-right` Display the next buffer in window to the right of the current one.
- · `windmove-display-same-window` Display the next buffer in the same window.
- · `windmove-display-up` Display the next buffer in window above the current one.
- · `windmove-down` Select the window below the current one.
- · `windmove-left` Select the window to the left of the current one.
- · `windmove-mode` Global minor mode for default windmove commands.
- · `windmove-right` Select the window to the right of the current one.
- · `windmove-swap-states-default-keybindings` Set up keybindings for directional window swap states.
- · `windmove-swap-states-down` Swap the states with the window below from the current one.
- · `windmove-swap-states-left` Swap the states with the window on the left from the current one.
- · `windmove-swap-states-right` Swap the states with the window on the right from the current one.
- · `windmove-swap-states-up` Swap the states with the window above from the current one.
- · `windmove-up` Select the window above the current one.

## window

- · `window-configuration-to-register` Store the window configuration of the selected frame in REGISTER.
- · `window-divider-mode` Display dividers between windows (Window Divider mode).
- · `window-layout-flip-leftright` Flip WINDOW's child windows horizontally.
- · `window-layout-flip-topdown` Flip WINDOW's child windows vertically.
- · `window-layout-rotate-anticlockwise` Rotate layout of WINDOW's child windows counterclockwise by 90 degrees.
- · `window-layout-rotate-clockwise` Rotate layout of WINDOW's child windows clockwise by 90 degrees.
- · `window-layout-transpose` Transpose child windows of WINDOW.
- · `window-swap-states` Swap the states of live windows WINDOW-1 and WINDOW-2.
- · `window-toggle-side-windows` Toggle display of side windows on specified FRAME.
- · `window-tool-bar-mode` Toggle display of the tool bar in the tab line of the current buffer.

## winner

- · `winner-mode` Toggle Winner mode on or off.

## wisent

- · `wisent-grammar-mode` Major mode for editing Wisent grammars.

## woman

- · `woman` Browse UN*X man page for TOPIC (Without using external Man program).
- · `woman-dired-find-file` In Dired, run the WoMan man-page browser on this file.
- · `woman-find-file` Find, decode and browse a specific UN*X man-page source file FILE-NAME.

## word

- · `word-search-backward` Search backward from point for STRING, ignoring differences in punctuation.
- · `word-search-backward-lax` Search backward from point for STRING, ignoring differences in punctuation.
- · `word-search-forward` Search forward from point for STRING, ignoring differences in punctuation.
- · `word-search-forward-lax` Search forward from point for STRING, ignoring differences in punctuation.
- · `word-wrap-whitespace-mode` Allow `word-wrap' to fold on all breaking whitespace characters.

## world

- · `world-clock` Display a world clock buffer with times in various time zones.

## write

- · `write-abbrev-file` Write all user-level abbrev definitions to a file of Lisp code.
- ✓ `write-file` Write current buffer into file FILENAME.
- · `write-region` Write current region into specified file.

## x

- · `x-clipboard-yank` Insert the clipboard contents, or the last stretch of killed text.
- · `x-dnd-handle-drag-n-drop-event` Receive drag and drop events (X client messages).
- · `x-menu-bar-open` Open the menu bar if it is shown.
- · `x-menu-bar-open-internal` Start key navigation of the menu bar in FRAME.
- · `x-preedit-text` Display preedit text from a compose sequence in EVENT.
- · `x-print-frames-dialog` Pop up a print dialog to print the current contents of FRAMES.
- · `x-set-selection` Make an X selection of type TYPE and value DATA.

## xdb

- · `xdb` Run xdb on program FILE in buffer *gud-FILE*.

## xml

- · `xml-mode` Major mode for editing XML.

## xref

- ✓ `xref-find-apropos` Find all meaningful symbols that match PATTERN.
- ✓ `xref-find-definitions` Find the definition of the identifier at point.
- · `xref-find-definitions-at-mouse` Find the definition of identifier at or around mouse click.
- · `xref-find-definitions-other-frame` Like `xref-find-definitions' but switch to the other frame.
- ✓ `xref-find-definitions-other-window` Like `xref-find-definitions' but switch to the other window.
- ✓ `xref-find-references` Find references to the identifier at point.
- · `xref-find-references-at-mouse` Find references to the identifier at or around mouse click.
- ✓ `xref-go-back` Go back to the previous position in xref history.
- ✓ `xref-go-forward` Go to the point where a previous \[xref-go-back] was invoked.
- · `xref-mouse-mode` Minor mode to bind Xref commands invoked using the mouse.
- · `xref-pop-marker-stack` Go back to the previous position in xref history.

## xterm

- · `xterm-mouse-mode` Toggle XTerm mouse mode.

## xwidget

- · `xwidget-webkit-browse-url` Ask xwidget-webkit to browse URL.

## y

- · `y-or-n-p-insert-n` Insert the answer "n" and exit the minibuffer of `y-or-n-p'.
- · `y-or-n-p-insert-other` Handle inserting of other answers in the minibuffer of `y-or-n-p'.
- · `y-or-n-p-insert-y` Insert the answer "y" and exit the minibuffer of `y-or-n-p'.

## yaml

- · `yaml-ts-mode` Major mode for editing YAML, powered by tree-sitter.

## yank

- ✓ `yank` Reinsert ("paste") the last stretch of killed text.
- · `yank-from-kill-ring` Select a stretch of previously killed text and insert ("paste") it.
- · `yank-in-context` Insert the last stretch of killed text while preserving syntax.
- · `yank-media` Yank media (images, HTML and the like) from the clipboard.
- ✓ `yank-pop` Replace just-yanked stretch of killed text with a different stretch.
- ✓ `yank-rectangle` Yank the last killed rectangle with upper left corner at point.

## yenc

- · `yenc-decode-region` Yenc decode region between START and END using an internal decoder.

## zap

- ✓ `zap-to-char` Kill up to and including ARGth occurrence of CHAR.
- ✓ `zap-up-to-char` Kill up to, but not including ARGth occurrence of CHAR.

## zone

- · `zone` Zone out, completely.
- · `zone-mode` Major mode for viewing and editing DNS master files.

## zrgrep

- · `zrgrep` Recursively grep for REGEXP in gzipped FILES in tree rooted at DIR.
