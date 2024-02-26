" Vim syntax file
" Language: Timesheets
" Maintainer: Julian Wiesler

if exists("b:current_syntax")
  finish
endif

syn match comment "#.*$"

syn match time "\d\d:\d\d" nextgroup=project skipwhite
syn match project "[[:alnum:]]\+" nextgroup=description skipwhite contained display
syn match description '.*$' contained display

syn match day '\*[^#]*'

let b:current_syntax = "tsh"

hi def link comment        Comment
hi def link project        Type
hi def link time           Constant
hi def link day            Statement
