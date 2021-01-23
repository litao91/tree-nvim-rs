"=============================================================================
" FILE: util.vim
" AUTHOR: Shougo Matsushita <Shougo.Matsu at gmail.com>
" License: MIT license
"=============================================================================
lua require 'tree'

function! tree#util#execute_path(command, path) abort
  try
    execute a:command fnameescape(v:lua.__expand(a:path))
  catch /^Vim\%((\a\+)\)\=:E325/
    " Ignore swap file error
  catch
    call v:lua.tree.print_error(v:throwpoint)
    call v:lua.tree.print_error(v:exception)
  endtry
endfunction

function! tree#util#cd(path) abort
  if exists('*chdir')
    call chdir(a:path)
  else
    silent execute (haslocaldir() ? 'lcd' : 'cd') fnameescape(a:path)
  endif
endfunction

function! tree#util#input(prompt, ...) abort
  let text = get(a:000, 0, '')
  let completion = get(a:000, 1, '')
  try
    if completion !=# ''
      return input(a:prompt, text, completion)
    else
      return input(a:prompt, text)
    endif
  catch
    " ignore the errors
    return ''
  endtry
endfunction

function! tree#util#confirm(msg, choices, default) abort
  try
    return confirm(a:msg, a:choices, a:default)
  catch
    " ignore the errors
  endtry

  return a:default
endfunction
