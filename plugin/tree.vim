if exists('g:loaded_tree')
    finish
endif
let g:loaded_tree = 1

command! -nargs=* -range -bar -complete=customlist,tree#util#complete
      \ Tree
      \ call tree#util#call_tree('Tree', <q-args>)
