#!/usr/bin/env bash
# Bash completion script for mml2vgm-rs
# Install with: sudo install -Dm644 completions/mml2vgm-rs.bash /usr/share/bash-completion/completions/mml2vgm-rs

_mml2vgm_rs() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    # Options available
    opts="--output --format --play --export-wav --verbose --quiet --debug --trace --check --list-chips --list-formats --chip --clock-count --include --batch --watch --progress --no-color --version --help -o -f -p -w -v -q -c -I -h"
    
    # Complete options
    if [[ ${cur} == -* ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
    fi
    
    # Complete format values after -f/--format
    if [[ ${prev} == "-f" || ${prev} == "--format" ]]; then
        local formats="vgm xgm xgm2 zgm mid"
        COMPREPLY=( $(compgen -W "${formats}" -- ${cur}) )
        return 0
    fi
    
    # Complete filenames for input/output options
    if [[ ${prev} == "-o" || ${prev} == "--output" || ${prev} == "-w" || ${prev} == "--export-wav" ]]; then
        COMPREPLY=( $(compgen -f -- ${cur}) )
        return 0
    fi
    
    # Complete chip values after -c/--chip
    if [[ ${prev} == "-c" || ${prev} == "--chip" ]]; then
        local chips="YM2612 YM2612X YM2612X2 SN76489 SN76489X2 AY8910 YM2608 YM2609 YM2610B YM2203 YM2151 YM3526 Y8950 YM3812 YMF262 YM2413 RF5C164 SegaPCM C140 C352 QSound HuC6280 K051649 K053260 K054539 NES DMG VRC6 POKEY MIDI"
        COMPREPLY=( $(compgen -W "${chips}" -- ${cur}) )
        return 0
    fi
    
    # Complete directory paths for batch/watch
    if [[ ${prev} == "--batch" || ${prev} == "--watch" ]]; then
        COMPREPLY=( $(compgen -d -- ${cur}) )
        return 0
    fi
    
    # Complete include paths
    if [[ ${prev} == "-I" || ${prev} == "--include" ]]; then
        COMPREPLY=( $(compgen -d -- ${cur}) )
        return 0
    fi
    
    # Complete .gwi files for input
    COMPREPLY=( $(compgen -f -o plusdirs -X '!*.gwi' -- ${cur}) )
    return 0
}

complete -o bashdefault -o default -o nospace -F _mml2vgm_rs mml2vgm-rs
