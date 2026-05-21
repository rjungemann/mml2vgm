" Vim syntax file for mml2vgm score files (.gwi, .muc, .mdl, .mus)
" Language:  mml2vgm MML
" Maintainer: mml2vgm Team

if exists("b:current_syntax")
  finish
endif

" Case-insensitive matching for commands
syn case ignore

" ── Comments ────────────────────────────────────────────────────────────────
syn match mmlComment ";.*$" contains=@Spell
hi def link mmlComment Comment

" ── Song info block  '{ ... } ───────────────────────────────────────────────
syn region mmlInfoBlock
      \ start="'{"
      \ end="}"
      \ contains=mmlInfoKey,mmlInfoEquals,mmlInfoValue,mmlComment
      \ fold

syn keyword mmlInfoKey
      \ TitleName Composer NoteAuthor SystemName Format ClockCount Octave-Rev
      \ contained

" Part assignments: PartYM2612, PartSN76489, etc.
syn match mmlInfoKey "\bPart[A-Za-z0-9]\+\b" contained

" Loop directives: LoopStart, LoopEnd, etc.
syn match mmlInfoKey "\bLoop[A-Za-z0-9]*\b" contained

syn match mmlInfoEquals "=" contained
syn match mmlInfoValue "=\s*\zs[^;}\n]\+" contained

hi def link mmlInfoBlock   Special
hi def link mmlInfoKey     Keyword
hi def link mmlInfoEquals  Operator
hi def link mmlInfoValue   String

" ── Instrument headers  '@ M/F/X4/R NNN "name" ──────────────────────────────
syn match mmlInstrHeader
      \ "'^@\s\+\(X[1-4]\|[MFR]\)\s\+[0-9]\+\(\s\+\"[^\"]*\"\)\?"
      \ contains=mmlInstrMode,mmlInstrNumber,mmlInstrName

syn match mmlInstrMode    "\(X[1-4]\|[MFR]\)" contained
syn match mmlInstrNumber  "[0-9]\+"           contained
syn region mmlInstrName   start=+"+ end=+"+ contained

hi def link mmlInstrHeader  Function
hi def link mmlInstrMode    StorageClass
hi def link mmlInstrNumber  Number
hi def link mmlInstrName    String

" ── Instrument parameter rows  '@ 031,010,... ────────────────────────────────
syn match mmlInstrRow "'^@\s\+[0-9][0-9, \t]\+" contains=mmlParamNum
syn match mmlParamNum "[0-9]\+" contained
hi def link mmlInstrRow  Type
hi def link mmlParamNum  Number

" ── Part declarations  'A1, 'B4, 'Vf01, etc. ─────────────────────────────────
syn match mmlPart "'^[A-Za-z][A-Za-z0-9]*"
hi def link mmlPart Statement

" ── Tempo / timing ───────────────────────────────────────────────────────────
syn match mmlTempo "[Tt][0-9]\+"
hi def link mmlTempo PreProc

" ── Instrument select  @N ─────────────────────────────────────────────────────
syn match mmlInstSel "@[0-9]\+"
hi def link mmlInstSel Identifier

" ── Volume  vN ───────────────────────────────────────────────────────────────
syn match mmlVolume "[Vv][0-9]\+"
hi def link mmlVolume Identifier

" ── Default note length  lN ──────────────────────────────────────────────────
syn match mmlLength "[Ll][0-9]\+"
hi def link mmlLength Identifier

" ── Octave  oN, < , > ────────────────────────────────────────────────────────
syn match mmlOctave "[Oo][0-9]\+"
syn match mmlOctaveShift "[<>]"
hi def link mmlOctave       Constant
hi def link mmlOctaveShift  Constant

" ── Gate / quantize  QN ──────────────────────────────────────────────────────
syn match mmlGate "[Qq][0-9]\+"
hi def link mmlGate Identifier

" ── Detune / pitch  D, P ──────────────────────────────────────────────────────
syn match mmlDetune "[Dd][+-]\?[0-9]\+"
syn match mmlPitch  "[Pp][+-]\?[0-9]\+"
hi def link mmlDetune Special
hi def link mmlPitch  Special

" ── Notes and rests ───────────────────────────────────────────────────────────
syn match mmlNote "[a-gA-G][+\-#]\?[0-9]*\.\?"
syn match mmlRest "[Rr][0-9]*\.\?"
hi def link mmlNote   Type
hi def link mmlRest   Comment

" ── Repeat blocks  [ ... ]N ──────────────────────────────────────────────────
syn match mmlRepeatStart "\["
syn match mmlRepeatEnd   "\][0-9]*"
hi def link mmlRepeatStart Delimiter
hi def link mmlRepeatEnd   Delimiter

let b:current_syntax = "mml2vgm"
