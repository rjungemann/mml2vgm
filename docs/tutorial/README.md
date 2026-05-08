# mml2vgm Tutorial

A step-by-step guide to composing chiptune music with mml2vgm, modeled after
[pedipanol's PMD guide](https://mml-guide.readthedocs.io/pmd/intro/).

Each page is a standalone document. Read them in order for the best experience,
or jump to any page if you already know the basics.

---

## Pages

| # | Page | Topic |
|---|------|-------|
| 1 | [Introduction](01_introduction.md) | What mml2vgm is, supported chips, the three tools |
| 2 | [Setting Up](02_setting_up.md) | Browser IDE, CLI (mml2vgm-rs), and egui desktop app |
| 3 | [Your First Song](03_your_first_song.md) | Step-by-step walkthrough of a complete `.gwi` file |
| 4 | [File Structure](04_file_structure.md) | Every section a `.gwi` file can contain |
| 5 | [Basic Sequencing](05_basic_sequencing.md) | Notes, lengths, octaves, tempo, volume, loops |
| 6 | [FM Synthesis Basics](06_fm_synthesis_basics.md) | 4-op FM theory and the instrument block |
| 7 | [PSG Channels](07_psg_channels.md) | SN76489 and AY8910 square-wave channels |
| 8 | [PCM Samples](08_pcm_samples.md) | Embedding WAV files as PCM instruments |
| 9 | [Envelopes and Arpeggios](09_envelopes_and_arpeggios.md) | Volume envelopes and pitch arpeggios |
| 10 | [Multi-chip Songs](10_multi_chip_songs.md) | Combining multiple chips in one song |
| 11 | [Tips and Tricks](11_tips_and_tricks.md) | Practical patterns, aliases, includes, workflow |
| 12 | [Resources](12_resources.md) | Further reading, VGM players, community links |

---

## Example Files

Working `.gwi` files for each tutorial page are in
[`docs/tutorial-examples/`](../tutorial-examples/).

| File | Corresponds To |
|------|---------------|
| [`01_hello.gwi`](../tutorial-examples/01_hello.gwi) | Page 3 — Your First Song |
| [`02_sequencing.gwi`](../tutorial-examples/02_sequencing.gwi) | Page 5 — Basic Sequencing |
| [`03_fm_scale.gwi`](../tutorial-examples/03_fm_scale.gwi) | Page 6 — FM Synthesis Basics |
| [`04_psg_demo.gwi`](../tutorial-examples/04_psg_demo.gwi) | Page 7 — PSG Channels |
| [`05_multi_chip.gwi`](../tutorial-examples/05_multi_chip.gwi) | Page 10 — Multi-chip Songs |
| [`06_envelopes.gwi`](../tutorial-examples/06_envelopes.gwi) | Page 9 — Envelopes and Arpeggios |

---

## Quick Start

If you just want to hear something right now, open
[`docs/tutorial-examples/01_hello.gwi`](../tutorial-examples/01_hello.gwi) in
the Browser IDE or run:

```sh
mml2vgm-rs docs/tutorial-examples/01_hello.gwi --play
```

Then read [Page 3 — Your First Song](03_your_first_song.md) to understand what
each line does.
