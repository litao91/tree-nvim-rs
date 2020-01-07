function! tree#initialize() abort
  return tree#init#_initialize()
endfunction

function! tree#start(paths, user_context) abort
  call tree#initialize()
  let context = tree#init#_context(a:user_context)
  let paths = a:paths
  let paths = map(paths, "fnamemodify(v:val, ':p')")
  if len(paths) == 0
    let paths = [expand('%:p:h')]
  endif
  call tree#util#rpcrequest('_tree_start', [paths, context], v:false)
endfunction
