# isend

![Build](https://github.com/machengim/isend/workflows/Build/badge.svg) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/machengim/isend/blob/master/LICENSE-MIT) ![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/machengim/isend)



Send files over LAN. Inspired by [Send-anywhere](https://send-anywhere.com/#transfer).

Compiled under Rust 1.47.

### Quick start:
    // Sender side
    isend -s -m "Hello from isend" a.txt b.png ~/Documents/

    // Receiver side
    isend -r your_code

### Help:

    isend --help
