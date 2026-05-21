# mml2vgm Vim / Neovim Syntax

Provides syntax highlighting and filetype detection for mml2vgm score files
in Vim and Neovim.

## Files

| File | Purpose |
|------|---------|
| `ftdetect/mml2vgm.vim` | Associates `.gwi`, `.muc`, `.mdl`, `.mus` with the `mml2vgm` filetype |
| `syntax/mml2vgm.vim` | Syntax rules and highlight groups |

## Installation

### Manual

Copy the two subdirectories into your Vim runtime path:

```sh
# Vim
cp ftdetect/mml2vgm.vim  ~/.vim/ftdetect/
cp syntax/mml2vgm.vim    ~/.vim/syntax/

# Neovim (XDG config)
cp ftdetect/mml2vgm.vim  ~/.config/nvim/ftdetect/
cp syntax/mml2vgm.vim    ~/.config/nvim/syntax/
```

### vim-plug / lazy.nvim (from repo root)

Add the repository as a plugin pointed at the `editors/vim` subdirectory, or
symlink the files from a local clone:

```sh
ln -s /path/to/maltese/editors/vim/ftdetect/mml2vgm.vim ~/.vim/ftdetect/
ln -s /path/to/maltese/editors/vim/syntax/mml2vgm.vim   ~/.vim/syntax/
```

## Highlight groups

The syntax file links to standard Vim highlight groups so it works with any
colour scheme:

| Group | Links to | Covers |
|-------|----------|--------|
| `mmlComment` | `Comment` | `;` line comments |
| `mmlInfoKey` | `Keyword` | Song info keys (`TitleName`, `PartYM2612`, …) |
| `mmlInfoValue` | `String` | Song info values |
| `mmlInstrMode` | `StorageClass` | `M`, `F`, `X4`, … |
| `mmlInstrNumber` | `Number` | Instrument numbers |
| `mmlInstrRow` | `Type` | Parameter rows |
| `mmlPart` | `Statement` | Part declarations (`'A1`, `'Vf01`, …) |
| `mmlTempo` | `PreProc` | `T120` |
| `mmlInstSel` | `Identifier` | `@N` |
| `mmlNote` | `Type` | Notes `c d e f g a b` |
| `mmlRest` | `Comment` | Rests `r` |
| `mmlOctave` | `Constant` | `o4`, `<`, `>` |
| `mmlRepeatStart/End` | `Delimiter` | `[` … `]N` |
