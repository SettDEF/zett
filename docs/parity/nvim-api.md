# Neovim's API (vim.api) and functions (vim.fn)

Generated 2026-10-21 by scripts/parity.sh from NVIM v0.12.6, GNU Emacs 31.1 and zett 0.1.0.

What a plugin can call. zett's own plugin API is `zett.*` (`:help config`, `:help plugins`); this is what
it is measured against.

## vim.api (181)

- `nvim__buf_debug_extmarks`
- `nvim__buf_stats`
- `nvim__complete_set`
- `nvim__get_lib_dir`
- `nvim__get_runtime`
- `nvim__id`
- `nvim__id_array`
- `nvim__id_dict`
- `nvim__id_float`
- `nvim__inspect_cell`
- `nvim__invalidate_glyph_cache`
- `nvim__ns_get`
- `nvim__ns_set`
- `nvim__redraw`
- `nvim__runtime_inspect`
- `nvim__screenshot`
- `nvim__stats`
- `nvim__unpack`
- `nvim_buf_add_highlight`
- `nvim_buf_attach`
- `nvim_buf_call`
- `nvim_buf_clear_highlight`
- `nvim_buf_clear_namespace`
- `nvim_buf_create_user_command`
- `nvim_buf_del_extmark`
- `nvim_buf_del_keymap`
- `nvim_buf_del_mark`
- `nvim_buf_del_user_command`
- `nvim_buf_del_var`
- `nvim_buf_delete`
- `nvim_buf_get_changedtick`
- `nvim_buf_get_commands`
- `nvim_buf_get_extmark_by_id`
- `nvim_buf_get_extmarks`
- `nvim_buf_get_keymap`
- `nvim_buf_get_lines`
- `nvim_buf_get_mark`
- `nvim_buf_get_name`
- `nvim_buf_get_number`
- `nvim_buf_get_offset`
- `nvim_buf_get_option`
- `nvim_buf_get_text`
- `nvim_buf_get_var`
- `nvim_buf_is_loaded`
- `nvim_buf_is_valid`
- `nvim_buf_line_count`
- `nvim_buf_set_extmark`
- `nvim_buf_set_keymap`
- `nvim_buf_set_lines`
- `nvim_buf_set_mark`
- `nvim_buf_set_name`
- `nvim_buf_set_option`
- `nvim_buf_set_text`
- `nvim_buf_set_var`
- `nvim_buf_set_virtual_text`
- `nvim_call_dict_function`
- `nvim_call_function`
- `nvim_chan_send`
- `nvim_clear_autocmds`
- `nvim_cmd`
- `nvim_command`
- `nvim_command_output`
- `nvim_create_augroup`
- `nvim_create_autocmd`
- `nvim_create_buf`
- `nvim_create_namespace`
- `nvim_create_user_command`
- `nvim_del_augroup_by_id`
- `nvim_del_augroup_by_name`
- `nvim_del_autocmd`
- `nvim_del_current_line`
- `nvim_del_keymap`
- `nvim_del_mark`
- `nvim_del_user_command`
- `nvim_del_var`
- `nvim_echo`
- `nvim_err_write`
- `nvim_err_writeln`
- `nvim_eval`
- `nvim_eval_statusline`
- `nvim_exec`
- `nvim_exec2`
- `nvim_exec_autocmds`
- `nvim_feedkeys`
- `nvim_get_all_options_info`
- `nvim_get_autocmds`
- `nvim_get_chan_info`
- `nvim_get_color_by_name`
- `nvim_get_color_map`
- `nvim_get_commands`
- `nvim_get_context`
- `nvim_get_current_buf`
- `nvim_get_current_line`
- `nvim_get_current_tabpage`
- `nvim_get_current_win`
- `nvim_get_hl`
- `nvim_get_hl_by_id`
- `nvim_get_hl_by_name`
- `nvim_get_hl_id_by_name`
- `nvim_get_hl_ns`
- `nvim_get_keymap`
- `nvim_get_mark`
- `nvim_get_mode`
- `nvim_get_namespaces`
- `nvim_get_option`
- `nvim_get_option_info`
- `nvim_get_option_info2`
- `nvim_get_option_value`
- `nvim_get_proc`
- `nvim_get_proc_children`
- `nvim_get_runtime_file`
- `nvim_get_var`
- `nvim_get_vvar`
- `nvim_input`
- `nvim_input_mouse`
- `nvim_list_bufs`
- `nvim_list_chans`
- `nvim_list_runtime_paths`
- `nvim_list_tabpages`
- `nvim_list_uis`
- `nvim_list_wins`
- `nvim_load_context`
- `nvim_notify`
- `nvim_open_tabpage`
- `nvim_open_term`
- `nvim_open_win`
- `nvim_out_write`
- `nvim_parse_cmd`
- `nvim_parse_expression`
- `nvim_paste`
- `nvim_put`
- `nvim_replace_termcodes`
- `nvim_select_popupmenu_item`
- `nvim_set_current_buf`
- `nvim_set_current_dir`
- `nvim_set_current_line`
- `nvim_set_current_tabpage`
- `nvim_set_current_win`
- `nvim_set_decoration_provider`
- `nvim_set_hl`
- `nvim_set_hl_ns`
- `nvim_set_hl_ns_fast`
- `nvim_set_keymap`
- `nvim_set_option`
- `nvim_set_option_value`
- `nvim_set_var`
- `nvim_set_vvar`
- `nvim_strwidth`
- `nvim_tabpage_del_var`
- `nvim_tabpage_get_number`
- `nvim_tabpage_get_var`
- `nvim_tabpage_get_win`
- `nvim_tabpage_is_valid`
- `nvim_tabpage_list_wins`
- `nvim_tabpage_set_var`
- `nvim_tabpage_set_win`
- `nvim_ui_send`
- `nvim_win_call`
- `nvim_win_close`
- `nvim_win_del_var`
- `nvim_win_get_buf`
- `nvim_win_get_config`
- `nvim_win_get_cursor`
- `nvim_win_get_height`
- `nvim_win_get_number`
- `nvim_win_get_option`
- `nvim_win_get_position`
- `nvim_win_get_tabpage`
- `nvim_win_get_var`
- `nvim_win_get_width`
- `nvim_win_hide`
- `nvim_win_is_valid`
- `nvim_win_set_buf`
- `nvim_win_set_config`
- `nvim_win_set_cursor`
- `nvim_win_set_height`
- `nvim_win_set_hl_ns`
- `nvim_win_set_option`
- `nvim_win_set_var`
- `nvim_win_set_width`
- `nvim_win_text_height`

## vim.fn (659)

<SNR>11_LocalBrowse <SNR>11_VimEnter <SNR>12_GetManifest <SNR>12_GetManifestPath 
<SNR>12_GetOldManifestPaths <SNR>12_LoadRemotePlugins <SNR>1_LoadFTPlugin <SNR>2_LoadIndent 
<SNR>5_SynSet <SNR>9_DoMatchParen <SNR>9_Highlight_Matching_Pair <SNR>9_NoMatchParen 
<SNR>9_Remove_Matches MatchDisable MatchEnable NetUserPass abs acos add and api_info append 
appendbufline argc argidx arglistid argv asin assert_beeps assert_equal assert_equalfile 
assert_exception assert_fails assert_false assert_inrange assert_match assert_nobeep 
assert_notequal assert_notmatch assert_report assert_true atan atan2 blob2list browse browsedir 
bufadd bufexists buffer_exists buffer_name buffer_number buflisted bufload bufloaded bufname bufnr 
bufwinid bufwinnr byte2line byteidx byteidxcomp call ceil chanclose changenr chansend char2nr 
charclass charcol charidx chdir cindent clearmatches cmdcomplete_info col complete complete_add 
complete_check complete_info confirm copy cos cosh count ctxget ctxpop ctxpush ctxset ctxsize 
cursor debugbreak deepcopy delete deletebufline dictwatcheradd dictwatcherdel did_filetype 
diff_filler diff_hlID digraph_get digraph_getlist digraph_set digraph_setlist empty environ escape 
eval eventhandler executable execute exepath exists exp expand expandcmd extend extendnew feedkeys 
file_readable filecopy filereadable filewritable filter finddir findfile flatten flattennew 
float2nr floor fmod fnameescape fnamemodify foldclosed foldclosedend foldlevel foldtext 
foldtextresult foreach foreground fullcommand funcref function garbagecollect get getbufinfo 
getbufline getbufoneline getbufvar getcellwidths getchangelist getchar getcharmod getcharpos 
getcharsearch getcharstr getcmdcomplpat getcmdcompltype getcmdline getcmdpos getcmdprompt 
getcmdscreenpos getcmdtype getcmdwintype getcompletion getcompletiontype getcurpos getcursorcharpos 
getcwd getenv getfontname getfperm getfsize getftime getftype getjumplist getline getloclist 
getmarklist getmatches getmousepos getpid getpos getqflist getreg getreginfo getregion getregionpos 
getregtype getscriptinfo getstacktrace gettabinfo gettabvar gettabwinvar gettagstack gettext 
getwininfo getwinpos getwinposx getwinposy getwinvar glob glob2regpat globpath has has_key 
haslocaldir hasmapto highlightID highlight_exists histadd histdel histget histnr hlID hlexists 
hostname iconv id indent index indexof input inputdialog inputlist inputrestore inputsave 
inputsecret insert interrupt invert isabsolutepath isdirectory isinf islocked isnan items jobclose 
jobpid jobresize jobsend jobstart jobstop jobwait join json_decode json_encode keys keytrans 
last_buffer_nr len libcall libcallnr line line2byte lispindent list2blob list2str localtime log 
log10 luaeval map maparg mapcheck maplist mapnew mapset match matchadd matchaddpos matcharg 
matchbufline matchdelete matchend matchfuzzy matchfuzzypos matchlist matchstr matchstrlist 
matchstrpos max menu_get menu_info min mkdir mode msgpackdump msgpackparse nextnonblank nr2char 
nvim__buf_debug_extmarks nvim__buf_stats nvim__complete_set nvim__get_lib_dir nvim__get_runtime 
nvim__id nvim__id_array nvim__id_dict nvim__id_float nvim__inspect_cell 
nvim__invalidate_glyph_cache nvim__ns_get nvim__ns_set nvim__redraw nvim__runtime_inspect 
nvim__screenshot nvim__stats nvim__unpack nvim_buf_add_highlight nvim_buf_attach 
nvim_buf_clear_highlight nvim_buf_clear_namespace nvim_buf_create_user_command nvim_buf_del_extmark 
nvim_buf_del_keymap nvim_buf_del_mark nvim_buf_del_user_command nvim_buf_del_var nvim_buf_delete 
nvim_buf_get_changedtick nvim_buf_get_commands nvim_buf_get_extmark_by_id nvim_buf_get_extmarks 
nvim_buf_get_keymap nvim_buf_get_lines nvim_buf_get_mark nvim_buf_get_name nvim_buf_get_number 
nvim_buf_get_offset nvim_buf_get_option nvim_buf_get_text nvim_buf_get_var nvim_buf_is_loaded 
nvim_buf_is_valid nvim_buf_line_count nvim_buf_set_extmark nvim_buf_set_keymap nvim_buf_set_lines 
nvim_buf_set_mark nvim_buf_set_name nvim_buf_set_option nvim_buf_set_text nvim_buf_set_var 
nvim_buf_set_virtual_text nvim_call_dict_function nvim_call_function nvim_clear_autocmds nvim_cmd 
nvim_command nvim_command_output nvim_create_augroup nvim_create_autocmd nvim_create_buf 
nvim_create_namespace nvim_create_user_command nvim_del_augroup_by_id nvim_del_augroup_by_name 
nvim_del_autocmd nvim_del_current_line nvim_del_keymap nvim_del_mark nvim_del_user_command 
nvim_del_var nvim_echo nvim_err_write nvim_err_writeln nvim_eval nvim_eval_statusline nvim_exec 
nvim_exec2 nvim_exec_autocmds nvim_feedkeys nvim_get_all_options_info nvim_get_autocmds 
nvim_get_chan_info nvim_get_color_by_name nvim_get_color_map nvim_get_commands nvim_get_context 
nvim_get_current_buf nvim_get_current_line nvim_get_current_tabpage nvim_get_current_win 
nvim_get_hl nvim_get_hl_by_id nvim_get_hl_by_name nvim_get_hl_id_by_name nvim_get_hl_ns 
nvim_get_keymap nvim_get_mark nvim_get_mode nvim_get_namespaces nvim_get_option 
nvim_get_option_info nvim_get_option_info2 nvim_get_option_value nvim_get_proc 
nvim_get_proc_children nvim_get_runtime_file nvim_get_var nvim_get_vvar nvim_input nvim_input_mouse 
nvim_list_bufs nvim_list_chans nvim_list_runtime_paths nvim_list_tabpages nvim_list_uis 
nvim_list_wins nvim_load_context nvim_notify nvim_open_tabpage nvim_open_term nvim_open_win 
nvim_out_write nvim_parse_cmd nvim_parse_expression nvim_paste nvim_put nvim_replace_termcodes 
nvim_select_popupmenu_item nvim_set_current_buf nvim_set_current_dir nvim_set_current_line 
nvim_set_current_tabpage nvim_set_current_win nvim_set_hl nvim_set_hl_ns nvim_set_hl_ns_fast 
nvim_set_keymap nvim_set_option nvim_set_option_value nvim_set_var nvim_set_vvar nvim_strwidth 
nvim_tabpage_del_var nvim_tabpage_get_number nvim_tabpage_get_var nvim_tabpage_get_win 
nvim_tabpage_is_valid nvim_tabpage_list_wins nvim_tabpage_set_var nvim_tabpage_set_win nvim_ui_send 
nvim_win_close nvim_win_del_var nvim_win_get_buf nvim_win_get_config nvim_win_get_cursor 
nvim_win_get_height nvim_win_get_number nvim_win_get_option nvim_win_get_position 
nvim_win_get_tabpage nvim_win_get_var nvim_win_get_width nvim_win_hide nvim_win_is_valid 
nvim_win_set_buf nvim_win_set_config nvim_win_set_cursor nvim_win_set_height nvim_win_set_hl_ns 
nvim_win_set_option nvim_win_set_var nvim_win_set_width nvim_win_text_height or pathshorten 
perleval pow preinserted prevnonblank printf prompt_appendbuf prompt_getinput prompt_getprompt 
prompt_setcallback prompt_setinterrupt prompt_setprompt pum_getpos pumvisible py3eval pyeval 
pyxeval rand range readblob readdir readfile reduce reg_executing reg_recorded reg_recording 
reltime reltimefloat reltimestr remove rename repeat resolve reverse round rpcnotify rpcrequest 
rpcstart rpcstop rubyeval screenattr screenchar screenchars screencol screenpos screenrow 
screenstring search searchcount searchdecl searchpair searchpairpos searchpos serverlist 
serverstart serverstop setbufline setbufvar setcellwidths setcharpos setcharsearch setcmdline 
setcmdpos setcursorcharpos setenv setfperm setline setloclist setmatches setpos setqflist setreg 
settabvar settabwinvar settagstack setwinvar sha256 shellescape shiftwidth sign_define 
sign_getdefined sign_getplaced sign_jump sign_place sign_placelist sign_undefine sign_unplace 
sign_unplacelist simplify sin sinh slice sockconnect sort soundfold spellbadword spellsuggest split 
sqrt srand state stdioopen stdpath str2float str2list str2nr strcharlen strcharpart strchars 
strdisplaywidth strftime strgetchar stridx string strlen strpart strptime strridx strtrans 
strutf16len strwidth submatch substitute swapfilelist swapinfo swapname synID synIDattr synIDtrans 
synconcealed synstack system systemlist tabpagebuflist tabpagenr tabpagewinnr tagfiles taglist tan 
tanh tempname termopen test_garbagecollect_now test_write_list_log timer_info timer_pause 
timer_start timer_stop timer_stopall tolower toupper tr trim trunc type undofile undotree uniq 
utf16idx values virtcol virtcol2col visualmode wait wildmenumode wildtrigger win_execute 
win_findbuf win_getid win_gettype win_gotoid win_id2tabwin win_id2win win_move_separator 
win_move_statusline win_screenpos win_splitmove winbufnr wincol windowsversion winheight winlayout 
winline winnr winrestcmd winrestview winsaveview winwidth wordcount writefile xor
