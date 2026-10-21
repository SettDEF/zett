# Neovim's options in zett

Generated 2026-10-21 by scripts/parity.sh from NVIM v0.12.6, GNU Emacs 31.1 and zett 0.1.0.

199 ✓ do something in zett, 175 ◐ are kept by :set but do nothing yet, of 374 (`:set`, `zett.opt`, `zett.ft.<type>`; `:help settings`).

| | option | short | type | scope | default | what it does |
|---|---|---|---|---|---|---|
| ◐ | `aleph` | al | number | global | `224` |  |
| ◐ | `allowrevins` | ari | boolean | global | `false` | allow CTRL-_ in Insert mode |
| ◐ | `ambiwidth` | ambw | string | global | `single` | what to do with Unicode chars of ambiguous width |
| ◐ | `arabic` | arab | boolean | win | `false` | for Arabic as a default second language |
| ◐ | `arabicshape` | arshape | boolean | global | `true` | do shaping for Arabic characters |
| ✓ | `autochdir` | acd | boolean | global | `false` | change directory to the file in the current window |
| ◐ | `autocomplete` | ac | boolean | buf | `false` | enable automatic completion in insert mode |
| ◐ | `autocompletedelay` | acl | number | global | `0` | delay in msec before menu appears after typing |
| ◐ | `autocompletetimeout` | act | number | global | `80` | initial decay timeout for autocompletion algorithm |
| ✓ | `autoindent` | ai | boolean | buf | `true` | take indent for new line from previous line |
| ✓ | `autoread` | ar | boolean | buf | `true` | autom. read file when changed outside of Vim |
| ✓ | `autowrite` | aw | boolean | global | `false` | automatically write file if changed |
| ✓ | `autowriteall` | awa | boolean | global | `false` | as 'autowrite', but works with more commands |
| ◐ | `background` | bg | string | global | `dark` | "dark" or "light", used for highlight colors |
| ✓ | `backspace` | bs | string | global | `indent,eol,start` | how backspace works at start of line |
| ✓ | `backup` | bk | boolean | global | `false` | keep backup file after overwriting a file |
| ✓ | `backupcopy` | bkc | string | buf | `auto` | make backup as a copy, don't rename the file |
| ✓ | `backupdir` | bdir | string | global | `.,/home/caesar/.local/state/nvim/backup//` | list of directories for the backup file |
| ✓ | `backupext` | bex | string | global | `~` | extension used for the backup file |
| ✓ | `backupskip` | bsk | string | global | `/tmp/*` | no backup for files that match these patterns |
| ✓ | `belloff` | bo | string | global | `all` | do not ring the bell for these reasons |
| ✓ | `binary` | bin | boolean | buf | `false` | read/write/edit file in binary mode |
| ✓ | `bomb` |  | boolean | buf | `false` | prepend a Byte Order Mark to the file |
| ✓ | `breakat` | brk | string | global | ` 	!@*-+;:,./?` | characters that may cause a line break |
| ✓ | `breakindent` | bri | boolean | win | `false` | wrapped line repeats indent |
| ✓ | `breakindentopt` | briopt | string | win | `` | settings for 'breakindent' |
| ✓ | `browsedir` | bsdir | string | global | `` | which directory to start browsing in |
| ◐ | `bufhidden` | bh | string | buf | `` | what to do when buffer is no longer in window |
| ◐ | `buflisted` | bl | boolean | buf | `true` | whether the buffer shows up in the buffer list |
| ◐ | `buftype` | bt | string | buf | `` | special type of buffer |
| ◐ | `busy` |  | number | buf | `0` |  |
| ◐ | `casemap` | cmp | string | global | `internal,keepascii` | specifies how case of letters is changed |
| ◐ | `cdhome` | cdh | boolean | global | `true` | change directory to the home directory by ":cd" |
| ◐ | `cdpath` | cd | string | global | `,,` | list of directories searched with ":cd" |
| ✓ | `cedit` |  | string | global | `` | key used to open the command-line window |
| ◐ | `channel` |  | number | buf | `0` |  |
| ◐ | `charconvert` | ccv | string | global | `` | expression for character encoding conversion |
| ✓ | `chistory` | chi | number | global | `10` | maximum number of quickfix lists in history |
| ✓ | `cindent` | cin | boolean | buf | `false` | do C program indenting |
| ◐ | `cinkeys` | cink | string | buf | `0{,0},0),0],:,0#,!^F,o,O,e` | keys that trigger indent when 'cindent' is set |
| ◐ | `cinoptions` | cino | string | buf | `` | how to do indenting when 'cindent' is set |
| ◐ | `cinscopedecls` | cinsd | string | buf | `public,protected,private` | words that are recognized by 'cino-g' |
| ◐ | `cinwords` | cinw | string | buf | `if,else,while,do,for,switch` | words where 'si' and 'cin' add an indent |
| ✓ | `clipboard` | cb | string | global | `` | use the clipboard as the unnamed register |
| ✓ | `cmdheight` | ch | number | global | `1` | number of lines to use for the command-line |
| ◐ | `cmdwinheight` | cwh | number | global | `7` | height of the command-line window |
| ✓ | `colorcolumn` | cc | string | win | `` | columns to highlight |
| ◐ | `columns` | co | number | global | `80` | number of columns in the display |
| ◐ | `comments` | com | string | buf | `s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-,fb:•` | patterns that can start a comment line |
| ✓ | `commentstring` | cms | string | buf | `` | template for comments; used for fold marker |
| ✓ | `compatible` | cp | boolean | global | `false` |  |
| ◐ | `complete` | cpt | string | buf | `.,w,b,u,t` | specify how Insert mode completion works |
| ◐ | `completefunc` | cfu | string | buf | `` | function to be used for Insert mode completion |
| ◐ | `completeitemalign` | cia | string | global | `abbr,kind,menu` |  |
| ◐ | `completeopt` | cot | string | buf | `menu,popup` | options for Insert mode completion |
| ✓ | `completeslash` | csl | string | buf | `` | like 'shellslash' for completion |
| ◐ | `completetimeout` | cto | number | global | `0` | initial decay timeout for CTRL-N and CTRL-P |
| ◐ | `concealcursor` | cocu | string | win | `` | whether concealable text is hidden in cursor line |
| ◐ | `conceallevel` | cole | number | win | `0` | whether concealable text is shown or hidden |
| ◐ | `confirm` | cf | boolean | global | `false` | ask what to do about unsaved/read-only files |
| ✓ | `copyindent` | ci | boolean | buf | `false` | make 'autoindent' use existing indent structure |
| ◐ | `cpoptions` | cpo | string | global | `aABceFs_` | flags for Vi-compatible behavior |
| ✓ | `cursorbind` | crb | boolean | win | `false` | move cursor in window as it moves in other windows |
| ✓ | `cursorcolumn` | cuc | boolean | win | `false` | highlight the screen column of the cursor |
| ✓ | `cursorline` | cul | boolean | win | `false` | highlight the screen line of the cursor |
| ✓ | `cursorlineopt` | culopt | string | win | `both` | settings for 'cursorline' |
| ◐ | `debug` |  | string | global | `` | set to "msg" to see all error messages |
| ◐ | `define` | def | string | buf | `` | pattern to be used to find a macro definition |
| ◐ | `delcombine` | deco | boolean | global | `false` | delete combining characters on their own |
| ◐ | `dictionary` | dict | string | buf | `` | list of file names used for keyword completion |
| ◐ | `diff` |  | boolean | win | `false` | use diff mode for the current window |
| ◐ | `diffanchors` | dia | string | buf | `` | list of {address} to force anchoring of a diff |
| ◐ | `diffexpr` | dex | string | global | `` | expression used to obtain a diff file |
| ◐ | `diffopt` | dip | string | global | `internal,filler,closeoff,indent-heuristic,inline:char,linematch:40` | options for using diff mode |
| ◐ | `digraph` | dg | boolean | global | `false` | enable the entering of digraphs in Insert mode |
| ◐ | `directory` | dir | string | global | `/home/caesar/.local/state/nvim/swap//` | list of directory names for the swap file |
| ◐ | `display` | dy | string | global | `lastline` | list of flags for how to display text |
| ✓ | `eadirection` | ead | string | global | `both` | in which direction 'equalalways' works |
| ✓ | `edcompatible` | ed | boolean | global | `false` |  |
| ◐ | `emoji` | emo | boolean | global | `true` |  |
| ✓ | `encoding` | enc | string | global | `utf-8` | encoding used internally |
| ◐ | `endoffile` | eof | boolean | buf | `false` | write CTRL-Z at end of the file |
| ✓ | `endofline` | eol | boolean | buf | `true` | write <EOL> for last line in file |
| ✓ | `equalalways` | ea | boolean | global | `true` | windows are automatically made the same size |
| ✓ | `equalprg` | ep | string | buf | `` | external program to use for "=" command |
| ✓ | `errorbells` | eb | boolean | global | `false` | ring the bell for error messages |
| ✓ | `errorfile` | ef | string | global | `errors.err` | name of the errorfile for the QuickFix mode |
| ✓ | `errorformat` | efm | string | buf | `%*[^"]"%f"%*\D%l: %m,"%f"%*\D%l: %m,%-Gg%\?make[%*\d]: *** [%f:%l:%m,%-Gg%\?make: *** [%f:%l:%m,%-G%f:%l: (Each undeclared identifier is reported only once,%-G%f:%l: for each function it appears in.),%-GIn file included from %f:%l:%c:,%-GIn file included from %f:%l:%c\,,%-GIn file included from %f:%l:%c,%-GIn file included from %f:%l,%-G%*[ ]from %f:%l:%c,%-G%*[ ]from %f:%l:,%-G%*[ ]from %f:%l\,,%-G%*[ ]from %f:%l,%f:%l:%c:%m,%f(%l):%m,%f:%l:%m,"%f"\, line %l%*\D%c%*[^ ] %m,%D%*\a[%*\d]: Entering directory %*[`']%f',%X%*\a[%*\d]: Leaving directory %*[`']%f',%D%*\a: Entering directory %*[`']%f',%X%*\a: Leaving directory %*[`']%f',%DMaking %*\a in %f,%f\|%l\| %m` | description of the lines in the error file |
| ◐ | `eventignore` | ei | string | global | `` | autocommand events that are ignored |
| ◐ | `eventignorewin` | eiw | string | win | `` | autocommand events that are ignored in a window |
| ✓ | `expandtab` | et | boolean | buf | `false` | use spaces when <Tab> is inserted |
| ◐ | `exrc` | ex | boolean | global | `false` | read init files in the current directory |
| ✓ | `fileencoding` | fenc | string | buf | `` | file encoding for multibyte text |
| ◐ | `fileencodings` | fencs | string | global | `ucs-bom,utf-8,default,latin1` | automatically detected character encodings |
| ✓ | `fileformat` | ff | string | buf | `unix` | file format used for file I/O |
| ◐ | `fileformats` | ffs | string | global | `unix,dos` | automatically detected values for 'fileformat' |
| ✓ | `fileignorecase` | fic | boolean | global | `false` | ignore case when using file names |
| ✓ | `filetype` | ft | string | buf | `` | type of file, used for autocommands |
| ✓ | `fillchars` | fcs | string | win | `` | characters to use for displaying special items |
| ◐ | `findfunc` | ffu | string | buf | `` | function to be called for the \|:find\| command |
| ✓ | `fixendofline` | fixeol | boolean | buf | `true` | make sure last line in file has <EOL> |
| ✓ | `foldclose` | fcl | string | global | `` | close a fold when the cursor leaves it |
| ✓ | `foldcolumn` | fdc | string | win | `0` | width of the column used to indicate folds |
| ✓ | `foldenable` | fen | boolean | win | `true` | set to display all folds open |
| ◐ | `foldexpr` | fde | string | win | `0` | expression used when 'foldmethod' is "expr" |
| ✓ | `foldignore` | fdi | string | win | `#` | ignore lines when 'foldmethod' is "indent" |
| ✓ | `foldlevel` | fdl | number | win | `0` | close folds with a level higher than this |
| ✓ | `foldlevelstart` | fdls | number | global | `-1` | 'foldlevel' when starting to edit a file |
| ✓ | `foldmarker` | fmr | string | win | `{{{,}}}` | markers used when 'foldmethod' is "marker" |
| ✓ | `foldmethod` | fdm | string | win | `manual` | folding type |
| ✓ | `foldminlines` | fml | number | win | `1` | minimum number of lines for a fold to be closed |
| ✓ | `foldnestmax` | fdn | number | win | `20` | maximum fold depth |
| ✓ | `foldopen` | fdo | string | global | `block,hor,mark,percent,quickfix,search,tag,undo` | for which commands a fold will be opened |
| ◐ | `foldtext` | fdt | string | win | `foldtext()` | expression used to display for a closed fold |
| ◐ | `formatexpr` | fex | string | buf | `` | expression used with "gq" command |
| ◐ | `formatlistpat` | flp | string | buf | `^\s*\d\+[\]:.)}\t ]\s*` | pattern used to recognize a list header |
| ✓ | `formatoptions` | fo | string | buf | `tcqj` | how automatic formatting is to be done |
| ✓ | `formatprg` | fp | string | buf | `` | name of external program used with "gq" command |
| ✓ | `fsync` | fs | boolean | buf | `true` | whether to invoke fsync() after file write |
| ✓ | `gdefault` | gd | boolean | global | `false` |  |
| ✓ | `grepformat` | gfm | string | buf | `%f:%l:%m,%f:%l%m,%f  %l%m` | format of 'grepprg' output |
| ✓ | `grepprg` | gp | string | buf | `grep -HIn $* /dev/null` | program to use for ":grep" |
| ✓ | `guicursor` | gcr | string | global | `n-v-c-sm:block,i-ci-ve:ver25,r-cr-o:hor20,t:block-blinkon500-blinkoff500-TermCursor` | GUI: settings for cursor shape and blinking |
| ✓ | `guifont` | gfn | string | global | `Source Code Pro,DejaVu Sans Mono,Courier New,monospace` | GUI: Name(s) of font(s) to be used |
| ✓ | `guifontwide` | gfw | string | global | `` | list of font names for double-wide characters |
| ✓ | `guioptions` | go | string | global | `` | GUI: Which components and options are used |
| ✓ | `guitablabel` | gtl | string | global | `` | GUI: custom label for a tabpage |
| ✓ | `guitabtooltip` | gtt | string | global | `` | GUI: custom tooltip for a tabpage |
| ◐ | `helpfile` | hf | string | global | `/usr/share/nvim/runtime/doc/help.txt` | full path name of the main help file |
| ◐ | `helpheight` | hh | number | global | `20` | minimum height of a new help window |
| ◐ | `helplang` | hlg | string | global | `` | preferred help languages |
| ✓ | `hidden` | hid | boolean | global | `true` | don't unload buffer when it is \|abandon\|ed |
| ◐ | `highlight` | hl | string | global | `8:SpecialKey,~:EndOfBuffer,z:TermCursor,@:NonText,d:Directory,e:ErrorMsg,i:IncSearch,l:Search,y:CurSearch,m:MoreMsg,M:ModeMsg,n:LineNr,a:LineNrAbove,b:LineNrBelow,N:CursorLineNr,G:CursorLineSign,O:CursorLineFold,r:Question,s:StatusLine,S:StatusLineNC,c:VertSplit,t:Title,v:Visual,V:VisualNOS,w:WarningMsg,W:WildMenu,f:Folded,F:FoldColumn,A:DiffAdd,C:DiffChange,D:DiffDelete,T:DiffText,E:DiffTextAdd,>:SignColumn,-:Conceal,B:SpellBad,P:SpellCap,R:SpellRare,L:SpellLocal,+:Pmenu,=:PmenuSel,k:PmenuMatch,<:PmenuMatchSel,[:PmenuKind,]:PmenuKindSel,{:PmenuExtra,}:PmenuExtraSel,x:PmenuSbar,X:PmenuThumb,*:TabLine,#:TabLineSel,_:TabLineFill,!:CursorColumn,.:CursorLine,o:ColorColumn,q:QuickFixLine,z:StatusLineTerm,Z:StatusLineTermNC,g:MsgArea,h:ComplMatchIns,0:Whitespace,I:PreInsert` |  |
| ✓ | `history` | hi | number | global | `10000` | number of command-lines that are remembered |
| ◐ | `hkmap` | hk | boolean | global | `false` |  |
| ◐ | `hkmapp` | hkp | boolean | global | `false` |  |
| ✓ | `hlsearch` | hls | boolean | global | `true` | highlight matches with last search pattern |
| ◐ | `icon` |  | boolean | global | `false` | let Vim set the text of the window icon |
| ◐ | `iconstring` |  | string | global | `` | string to use for the Vim icon text |
| ✓ | `ignorecase` | ic | boolean | global | `false` | ignore case in search patterns |
| ◐ | `imcmdline` | imc | boolean | global | `false` | use IM when starting to edit a command line |
| ◐ | `imdisable` | imd | boolean | global | `false` | do not use the IM in any mode |
| ◐ | `iminsert` | imi | number | buf | `0` | use :lmap or IM in Insert mode |
| ◐ | `imsearch` | ims | number | buf | `-1` | use :lmap or IM when typing a search pattern |
| ◐ | `inccommand` | icm | string | global | `nosplit` |  |
| ◐ | `include` | inc | string | buf | `` | pattern to be used to find an include file |
| ◐ | `includeexpr` | inex | string | buf | `` | expression used to process an include line |
| ✓ | `incsearch` | is | boolean | global | `true` | highlight match while typing search pattern |
| ◐ | `indentexpr` | inde | string | buf | `` | expression used to obtain the indent of a line |
| ◐ | `indentkeys` | indk | string | buf | `0{,0},0),0],:,0#,!^F,o,O,e` | keys that trigger indenting with 'indentexpr' |
| ◐ | `infercase` | inf | boolean | buf | `false` | adjust case of match for keyword completion |
| ✓ | `insertmode` | im | boolean | global | `false` |  |
| ◐ | `isfname` | isf | string | global | `@,48-57,/,.,-,_,+,,,#,$,%,~,=` | characters included in file names and pathnames |
| ◐ | `isident` | isi | string | global | `@,48-57,_,192-255` | characters included in identifiers |
| ✓ | `iskeyword` | isk | string | buf | `@,48-57,_,192-255` | characters included in keywords |
| ◐ | `isprint` | isp | string | global | `@,161-255` | printable characters |
| ✓ | `joinspaces` | js | boolean | global | `false` | two spaces after a period with a join command |
| ◐ | `jumpoptions` | jop | string | global | `clean` | specifies how jumping is done |
| ◐ | `keymap` | kmp | string | buf | `` | name of a keyboard mapping |
| ◐ | `keymodel` | km | string | global | `` | enable starting/stopping selection with keys |
| ◐ | `keywordprg` | kp | string | buf | `:Man` | program to use for the "K" command |
| ✓ | `langmap` | lmap | string | global | `` | alphabetic characters for other language mode |
| ✓ | `langmenu` | lm | string | global | `` | language to be used for the menus |
| ◐ | `langnoremap` | lnr | boolean | global | `true` |  |
| ◐ | `langremap` | lrm | boolean | global | `false` | do apply 'langmap' to mapped characters |
| ✓ | `laststatus` | ls | number | global | `2` | tells when last window has status lines |
| ◐ | `lazyredraw` | lz | boolean | global | `false` | don't redraw while executing macros |
| ✓ | `lhistory` | lhi | number | win | `10` | maximum number of location lists in history |
| ✓ | `linebreak` | lbr | boolean | win | `false` | wrap long lines at a blank |
| ◐ | `lines` |  | number | global | `24` | number of lines in the display |
| ✓ | `linespace` | lsp | number | global | `0` | number of pixel lines to use between characters |
| ◐ | `lisp` |  | boolean | buf | `false` | automatic indenting for Lisp |
| ◐ | `lispoptions` | lop | string | buf | `` | changes how Lisp indenting is done |
| ◐ | `lispwords` | lw | string | buf | `defun,define,defmacro,set!,lambda,if,case,let,flet,let*,letrec,do,do*,define-syntax,let-syntax,letrec-syntax,destructuring-bind,defpackage,defparameter,defstruct,deftype,defvar,do-all-symbols,do-external-symbols,do-symbols,dolist,dotimes,ecase,etypecase,eval-when,labels,macrolet,multiple-value-bind,multiple-value-call,multiple-value-prog1,multiple-value-setq,prog1,progv,typecase,unless,unwind-protect,when,with-input-from-string,with-open-file,with-open-stream,with-output-to-string,with-package-iterator,define-condition,handler-bind,handler-case,restart-bind,restart-case,with-simple-restart,store-value,use-value,muffle-warning,abort,continue,with-slots,with-slots*,with-accessors,with-accessors*,defclass,defmethod,print-unreadable-object` | words that change how lisp indenting works |
| ✓ | `list` |  | boolean | win | `false` | show <Tab> and <EOL> |
| ✓ | `listchars` | lcs | string | win | `tab:> ,trail:-,nbsp:+` | characters for displaying in list mode |
| ◐ | `loadplugins` | lpl | boolean | global | `true` | load plugin scripts when starting up |
| ◐ | `magic` |  | boolean | global | `true` |  |
| ◐ | `makeef` | mef | string | global | `` | name of the errorfile for ":make" |
| ✓ | `makeencoding` | menc | string | buf | `` | encoding of external make/grep commands |
| ✓ | `makeprg` | mp | string | buf | `make` | program to use for the ":make" command |
| ✓ | `matchpairs` | mps | string | buf | `(:),{:},[:]` | pairs of characters that "%" can match |
| ✓ | `matchtime` | mat | number | global | `5` | tenths of a second to show matching paren |
| ✓ | `maxcombine` | mco | number | global | `6` | maximum nr of combining characters displayed |
| ◐ | `maxfuncdepth` | mfd | number | global | `100` | maximum recursive depth for user functions |
| ◐ | `maxmapdepth` | mmd | number | global | `1000` | maximum recursive depth for mapping |
| ◐ | `maxmempattern` | mmp | number | global | `1000` | maximum memory (in Kbyte) used for pattern search |
| ✓ | `maxsearchcount` | msc | number | global | `999` |  |
| ✓ | `menuitems` | mis | number | global | `25` | maximum number of items in a menu |
| ✓ | `messagesopt` | mopt | string | global | `hit-enter,history:500,progress:c` |  |
| ◐ | `mkspellmem` | msm | string | global | `460000,2000,500` | memory used before \|:mkspell\| compresses the tree |
| ✓ | `modeline` | ml | boolean | buf | `true` | recognize modelines at start or end of file |
| ✓ | `modelineexpr` | mle | boolean | global | `false` | allow setting expression options from a modeline |
| ✓ | `modelines` | mls | number | global | `5` | number of lines checked for modelines |
| ✓ | `modifiable` | ma | boolean | buf | `true` | changes to the text are not possible |
| ✓ | `modified` | mod | boolean | buf | `false` | buffer has been modified |
| ◐ | `more` |  | boolean | global | `true` | pause listings when the whole screen is filled |
| ✓ | `mouse` |  | string | global | `nvi` | enable the use of mouse clicks |
| ◐ | `mousefocus` | mousef | boolean | global | `false` | keyboard focus follows the mouse |
| ✓ | `mousehide` | mh | boolean | global | `true` | hide mouse pointer while typing |
| ✓ | `mousemodel` | mousem | string | global | `popup_setpos` | changes meaning of mouse buttons |
| ◐ | `mousemoveevent` | mousemev | boolean | global | `false` | report mouse moves with <MouseMove> |
| ✓ | `mousescroll` |  | string | global | `ver:3,hor:6` | amount to scroll by when scrolling with a mouse |
| ✓ | `mouseshape` | mouses | string | global | `` | shape of the mouse pointer in different modes |
| ✓ | `mousetime` | mouset | number | global | `500` | max time between mouse double-click |
| ✓ | `nrformats` | nf | string | buf | `bin,hex` | number formats recognized for CTRL-A command |
| ✓ | `number` | nu | boolean | win | `false` | print the line number in front of each line |
| ✓ | `numberwidth` | nuw | number | win | `4` | number of columns used for the line number |
| ◐ | `omnifunc` | ofu | string | buf | `` | function for filetype-specific completion |
| ✓ | `opendevice` | odev | boolean | global | `false` | allow reading/writing devices on MS-Windows |
| ✓ | `operatorfunc` | opfunc | string | global | `` | function to be called for \|g@\| operator |
| ◐ | `packpath` | pp | string | global | `/etc/xdg/nvim,/home/caesar/.local/share/flatpak/exports/share/nvim/site,/var/lib/flatpak/exports/share/nvim/site,/usr/local/share/nvim/site,/usr/share/nvim/site,/home/caesar/.nix-profile/share/nvim/site,/nix/var/nix/profiles/default/share/nvim/site,/usr/share/nvim/runtime,/usr/lib/nvim,/nix/var/nix/profiles/default/share/nvim/site/after,/home/caesar/.nix-profile/share/nvim/site/after,/usr/share/nvim/site/after,/usr/local/share/nvim/site/after,/var/lib/flatpak/exports/share/nvim/site/after,/home/caesar/.local/share/flatpak/exports/share/nvim/site/after,/etc/xdg/nvim/after` | list of directories used for packages |
| ✓ | `paragraphs` | para | string | global | `IPLPPPQPP TPHPLIPpLpItpplpipbp` | nroff macros that separate paragraphs |
| ✓ | `paste` |  | boolean | global | `false` |  |
| ◐ | `pastetoggle` | pt | string | global | `` |  |
| ◐ | `patchexpr` | pex | string | global | `` | expression used to patch a file |
| ◐ | `patchmode` | pm | string | global | `` | keep the oldest version of a file |
| ◐ | `path` | pa | string | buf | `.,,` | list of directories searched with "gf" et.al. |
| ✓ | `preserveindent` | pi | boolean | buf | `false` | preserve the indent structure when reindenting |
| ◐ | `previewheight` | pvh | number | global | `12` | height of the preview window |
| ◐ | `previewwindow` | pvw | boolean | win | `false` | identifies the preview window |
| ◐ | `prompt` |  | boolean | global | `true` |  |
| ◐ | `pumblend` | pb | number | global | `0` |  |
| ◐ | `pumborder` |  | string | global | `` |  |
| ✓ | `pumheight` | ph | number | global | `0` | maximum number of items to show in the popup menu |
| ✓ | `pummaxwidth` | pmw | number | global | `0` |  |
| ✓ | `pumwidth` | pw | number | global | `15` | minimum width of the popup menu |
| ◐ | `pyxversion` | pyx | number | global | `3` | Python version used for pyx* commands |
| ◐ | `quickfixtextfunc` | qftf | string | global | `` |  |
| ✓ | `quoteescape` | qe | string | buf | `\` | escape characters used in a string |
| ✓ | `readonly` | ro | boolean | buf | `false` | disallow writing the buffer |
| ◐ | `redrawdebug` | rdb | string | global | `` |  |
| ◐ | `redrawtime` | rdt | number | global | `2000` | timeout for 'hlsearch' and \|:match\| highlighting |
| ◐ | `regexpengine` | re | number | global | `0` | default regexp engine to use |
| ✓ | `relativenumber` | rnu | boolean | win | `false` | show relative line number in front of each line |
| ◐ | `remap` |  | boolean | global | `true` |  |
| ✓ | `report` |  | number | global | `2` | threshold for reporting nr. of lines changed |
| ◐ | `revins` | ri | boolean | global | `false` | inserting characters will work backwards |
| ◐ | `rightleft` | rl | boolean | win | `false` | window is right-to-left oriented |
| ◐ | `rightleftcmd` | rlc | string | win | `search` | commands for which editing works right-to-left |
| ✓ | `ruler` | ru | boolean | global | `true` | show cursor line and column in the status line |
| ✓ | `rulerformat` | ruf | string | global | `` | custom format for the ruler |
| ◐ | `runtimepath` | rtp | string | global | `/etc/xdg/nvim,$HOME/.local/share/flatpak/exports/share/nvim/site,/var/lib/flatpak/exports/share/nvim/site,/usr/local/share/nvim/site,/usr/share/nvim/site,/home/caesar/.nix-profile/share/nvim/site,/nix/var/nix/profiles/default/share/nvim/site,/usr/share/nvim/runtime,/usr/lib/nvim,/nix/var/nix/profiles/default/share/nvim/site/after,/home/caesar/.nix-profile/share/nvim/site/after,/usr/share/nvim/site/after,/usr/local/share/nvim/site/after,/var/lib/flatpak/exports/share/nvim/site/after,$HOME/.local/share/flatpak/exports/share/nvim/site/after,/etc/xdg/nvim/after` | list of directories used for runtime files |
| ✓ | `scroll` | scr | number | win | `0` | lines to scroll with CTRL-U and CTRL-D |
| ◐ | `scrollback` | scbk | number | buf | `-1` |  |
| ✓ | `scrollbind` | scb | boolean | win | `false` | scroll in window as other windows scroll |
| ✓ | `scrolljump` | sj | number | global | `1` | minimum number of lines to scroll |
| ✓ | `scrolloff` | so | number | win | `0` | minimum nr. of lines above and below cursor |
| ✓ | `scrollopt` | sbo | string | global | `ver,jump` | how 'scrollbind' should behave |
| ✓ | `sections` | sect | string | global | `SHNHH HUnhsh` | nroff macros that separate sections |
| ◐ | `secure` |  | boolean | global | `false` | secure mode for reading .vimrc in current dir |
| ✓ | `selection` | sel | string | global | `inclusive` | what type of selection to use |
| ◐ | `selectmode` | slm | string | global | `` | when to use Select mode instead of Visual mode |
| ◐ | `sessionoptions` | ssop | string | global | `blank,buffers,curdir,folds,help,tabpages,winsize,terminal` | options for \|:mksession\| |
| ◐ | `shada` | sd | string | global | `!,'100,<50,s10,h,r/tmp/,r/private/` | use \|shada\| file upon startup and exiting |
| ◐ | `shadafile` | sdf | string | global | `` |  |
| ✓ | `shell` | sh | string | global | `/bin/zsh` | name of shell to use for external commands |
| ✓ | `shellcmdflag` | shcf | string | global | `-c` | flag to shell to execute one command |
| ◐ | `shellpipe` | sp | string | global | `2>&1\| tee` | string to put output of ":make" in error file |
| ✓ | `shellquote` | shq | string | global | `` | quote character(s) for around shell command |
| ◐ | `shellredir` | srr | string | global | `>%s 2>&1` | string to put output of filter in a temp file |
| ✓ | `shellslash` | ssl | boolean | global | `true` | use forward slash for shell file names |
| ◐ | `shelltemp` | stmp | boolean | global | `false` | whether to use a temp file for shell commands |
| ◐ | `shellxescape` | sxe | string | global | `` | characters to escape when 'shellxquote' is ( |
| ✓ | `shellxquote` | sxq | string | global | `` | like 'shellquote', but include redirection |
| ✓ | `shiftround` | sr | boolean | global | `false` | round indent to multiple of shiftwidth |
| ✓ | `shiftwidth` | sw | number | buf | `8` | number of spaces to use for (auto)indent step |
| ✓ | `shortmess` | shm | string | global | `ltToOCF` | list of flags, reduce length of messages |
| ✓ | `showbreak` | sbr | string | win | `` | string to use at the start of wrapped lines |
| ✓ | `showcmd` | sc | boolean | global | `true` | show (partial) command somewhere |
| ✓ | `showcmdloc` | sloc | string | global | `last` | where to show (partial) command |
| ◐ | `showfulltag` | sft | boolean | global | `false` | show full tag pattern when completing tag |
| ✓ | `showmatch` | sm | boolean | global | `false` | briefly jump to matching bracket if insert one |
| ✓ | `showmode` | smd | boolean | global | `true` | message on status line to show current mode |
| ✓ | `showtabline` | stal | number | global | `1` | tells when the tabpages line is displayed |
| ✓ | `sidescroll` | ss | number | global | `1` | minimum number of columns to scroll horizontal |
| ✓ | `sidescrolloff` | siso | number | win | `0` | min. nr. of columns to left and right of cursor |
| ✓ | `signcolumn` | scl | string | win | `auto` | when and how to display the sign column |
| ✓ | `smartcase` | scs | boolean | global | `false` | no ignore case when pattern has uppercase |
| ✓ | `smartindent` | si | boolean | buf | `false` | smart autoindenting for C programs |
| ✓ | `smarttab` | sta | boolean | global | `true` | <Tab> in leading whitespace indents by 'shiftwidth' |
| ◐ | `smoothscroll` | sms | boolean | win | `false` | scroll by screen lines when 'wrap' is set |
| ✓ | `softtabstop` | sts | number | buf | `0` | number of columns between two soft tab stops |
| ✓ | `spell` |  | boolean | win | `false` | enable spell checking |
| ✓ | `spellcapcheck` | spc | string | buf | `[.?!]\_[\])'"\t ]\+` | pattern to locate end of a sentence |
| ✓ | `spellfile` | spf | string | buf | `` | files where \|zg\| and \|zw\| store words |
| ✓ | `spelllang` | spl | string | buf | `en` | language(s) to do spell checking for |
| ✓ | `spelloptions` | spo | string | buf | `` | options for spell checking |
| ✓ | `spellsuggest` | sps | string | global | `best` | method(s) used to suggest spelling corrections |
| ✓ | `splitbelow` | sb | boolean | global | `false` | new window from split is below the current one |
| ◐ | `splitkeep` | spk | string | global | `cursor` | determines scroll behavior for split windows |
| ✓ | `splitright` | spr | boolean | global | `false` | new window is put right of the current one |
| ✓ | `startofline` | sol | boolean | global | `false` | commands move cursor to first non-blank in line |
| ◐ | `statuscolumn` | stc | string | win | `` | custom format for the status column |
| ✓ | `statusline` | stl | string | win | `%<%f %h%w%m%r %{% v:lua.require('vim._core.util').term_exitcode() %}%=%{% luaeval('(package.loaded[''vim.ui''] and vim.api.nvim_get_current_win() == tonumber(vim.g.actual_curwin or -1) and vim.ui.progress_status()) or '''' ')%}%{% &showcmdloc == 'statusline' ? '%-10.S ' : '' %}%{% exists('b:keymap_name') ? '<'..b:keymap_name..'> ' : '' %}%{% &busy > 0 ? '◐ ' : '' %}%{% luaeval('(package.loaded[''vim.diagnostic''] and next(vim.diagnostic.count()) and vim.diagnostic.status() .. '' '') or '''' ') %}%{% &ruler ? ( &rulerformat == '' ? '%-14.(%l,%c%V%) %P' : &rulerformat ) : '' %}` | custom format for the status line |
| ✓ | `suffixes` | su | string | global | `.bak,~,.o,.h,.info,.swp,.obj` | suffixes that are ignored with multiple match |
| ◐ | `suffixesadd` | sua | string | buf | `` | suffixes added when searching for a file |
| ◐ | `swapfile` | swf | boolean | buf | `true` | whether to use a swapfile for a buffer |
| ✓ | `switchbuf` | swb | string | global | `uselast` | sets behavior when switching to another buffer |
| ✓ | `synmaxcol` | smc | number | buf | `3000` | maximum column to find syntax items |
| ✓ | `syntax` | syn | string | buf | `` | syntax to be loaded for current buffer |
| ◐ | `tabclose` | tcl | string | global | `` | which tabpage to focus when closing a tab |
| ✓ | `tabline` | tal | string | global | `` | custom format for the console tabpages line |
| ◐ | `tabpagemax` | tpm | number | global | `50` | maximum number of tabpages for \|-p\| and "tab all" |
| ✓ | `tabstop` | ts | number | buf | `8` | number of columns between two tab stops |
| ◐ | `tagbsearch` | tbs | boolean | global | `true` | use binary searching in tags files |
| ◐ | `tagcase` | tc | string | buf | `followic` | how to handle case when searching in tags files |
| ◐ | `tagfunc` | tfu | string | buf | `` | function to get list of tag matches |
| ◐ | `taglength` | tl | number | global | `0` | number of significant characters for a tag |
| ◐ | `tagrelative` | tr | boolean | global | `true` | file names in tag file are relative |
| ◐ | `tags` | tag | string | buf | `./tags;,tags` | list of file names used by the tag command |
| ◐ | `tagstack` | tgst | boolean | global | `true` | push tags onto the tag stack |
| ◐ | `termbidi` | tbidi | boolean | global | `false` | terminal takes care of bi-directionality |
| ✓ | `termencoding` | tenc | string | global | `` |  |
| ✓ | `termguicolors` | tgc | boolean | global | `false` | enable 24-bit RGB color in the TUI |
| ◐ | `termpastefilter` | tpf | string | global | `BS,HT,ESC,DEL` |  |
| ✓ | `termsync` |  | boolean | global | `true` |  |
| ◐ | `terse` |  | boolean | global | `false` |  |
| ✓ | `textwidth` | tw | number | buf | `0` | maximum width of text that is being inserted |
| ◐ | `thesaurus` | tsr | string | buf | `` | list of thesaurus files for keyword completion |
| ◐ | `thesaurusfunc` | tsrfu | string | buf | `` | function to be used for thesaurus completion |
| ✓ | `tildeop` | top | boolean | global | `false` | tilde command "~" behaves like an operator |
| ✓ | `timeout` | to | boolean | global | `true` | time out on mappings and key codes |
| ✓ | `timeoutlen` | tm | number | global | `1000` | time out time in milliseconds |
| ✓ | `title` |  | boolean | global | `false` | let Vim set the title of the window |
| ✓ | `titlelen` |  | number | global | `85` | percentage of 'columns' used for window title |
| ◐ | `titleold` |  | string | global | `` | old title, restored when exiting |
| ✓ | `titlestring` |  | string | global | `` | string to use for the Vim window title |
| ◐ | `ttimeout` |  | boolean | global | `true` | time out on mappings |
| ◐ | `ttimeoutlen` | ttm | number | global | `50` | time out time for key codes in milliseconds |
| ✓ | `ttyfast` | tf | boolean | global | `true` |  |
| ◐ | `undodir` | udir | string | global | `/home/caesar/.local/state/nvim/undo//` | where to store undo files |
| ✓ | `undofile` | udf | boolean | buf | `false` | save undo information in a file |
| ◐ | `undolevels` | ul | number | buf | `1000` | maximum number of changes that can be undone |
| ◐ | `undoreload` | ur | number | global | `10000` | max nr of lines to save for undo on a buffer reload |
| ◐ | `updatecount` | uc | number | global | `200` | after this many characters flush swap file |
| ◐ | `updatetime` | ut | number | global | `4000` | after this many milliseconds flush swap file |
| ◐ | `varsofttabstop` | vsts | string | buf | `` | a list of number of columns between soft tab stops |
| ◐ | `vartabstop` | vts | string | buf | `` | a list of number of columns between tab stops |
| ◐ | `verbose` | vbs | number | global | `0` | give informative messages |
| ◐ | `verbosefile` | vfile | string | global | `` | file to write messages in |
| ◐ | `viewdir` | vdir | string | global | `/home/caesar/.local/state/nvim/view//` | directory where to store files with :mkview |
| ◐ | `viewoptions` | vop | string | global | `folds,cursor,curdir` | specifies what to save for :mkview |
| ◐ | `virtualedit` | ve | string | win | `` | when to use virtual editing |
| ✓ | `visualbell` | vb | boolean | global | `false` | use visual bell instead of beeping |
| ◐ | `warn` |  | boolean | global | `true` | warn for shell command when buffer was changed |
| ✓ | `whichwrap` | ww | string | global | `b,s` | allow specified keys to cross line boundaries |
| ◐ | `wildchar` | wc | number | global | `9` | command-line character for wildcard expansion |
| ◐ | `wildcharm` | wcm | number | global | `0` | like 'wildchar' but also works when mapped |
| ✓ | `wildignore` | wig | string | global | `` | files matching these patterns are not completed |
| ✓ | `wildignorecase` | wic | boolean | global | `false` | ignore case when completing file names |
| ✓ | `wildmenu` | wmnu | boolean | global | `true` | use menu for command line completion |
| ✓ | `wildmode` | wim | string | global | `full` | mode for 'wildchar' command-line expansion |
| ✓ | `wildoptions` | wop | string | global | `pum,tagfile` | specifies how command line completion is done |
| ✓ | `winaltkeys` | wak | string | global | `menu` | when the windows system handles ALT keys |
| ✓ | `winbar` | wbr | string | win | `` |  |
| ◐ | `winblend` | winbl | number | win | `0` |  |
| ✓ | `winborder` |  | string | global | `` |  |
| ◐ | `window` | wi | number | global | `23` | nr of lines to scroll for CTRL-F and CTRL-B |
| ✓ | `winfixbuf` | wfb | boolean | win | `false` | keep window focused on a single buffer |
| ◐ | `winfixheight` | wfh | boolean | win | `false` | keep window height when opening/closing windows |
| ◐ | `winfixwidth` | wfw | boolean | win | `false` | keep window width when opening/closing windows |
| ✓ | `winheight` | wh | number | global | `1` | minimum number of lines for the current window |
| ◐ | `winhighlight` | winhl | string | win | `` | window-local highlighting |
| ✓ | `winminheight` | wmh | number | global | `1` | minimum number of lines for any window |
| ✓ | `winminwidth` | wmw | number | global | `1` | minimal number of columns for any window |
| ✓ | `winwidth` | wiw | number | global | `20` | minimal number of columns for current window |
| ✓ | `wrap` |  | boolean | win | `true` | long lines wrap and continue on the next line |
| ✓ | `wrapmargin` | wm | number | buf | `0` | chars from the right where wrapping starts |
| ✓ | `wrapscan` | ws | boolean | global | `true` | searches wrap around the end of the file |
| ✓ | `write` |  | boolean | global | `true` | writing to a file is allowed |
| ◐ | `writeany` | wa | boolean | global | `false` | write to file with no need for "!" override |
| ✓ | `writebackup` | wb | boolean | global | `true` | make a backup before overwriting a file |
| ◐ | `writedelay` | wd | number | global | `0` | delay this many msec for each char (for debug) |
