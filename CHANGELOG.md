# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Note that, when a release point is hit where many changes are made, these changes are sorted, as best as possible, by date, with the newest changes first. This will prevent confusion if changes are reversed during the development cycle.

## [Unreleased]

### 2020-03-20

- Deps: update goblin to v. 0.2.1
- Page alloc: Use a static array of frames for backing storage of memory frames to speed up page frame allocation (currently a size of 65535 frames is used)
- Heap: search for a random memory address for the heap that is modulo 32767 instead of placing it at a constant address
- Malloc: swap memory allocator to use a linked list allocator instead of slabs
- Security: compile with retpoline mitigations (features: retpoline, retpoline-external-thunk, retpoline-indirect-branches, retpoline-indirect-calls)
- Misc: Add MMX, SSE, SSE2/3/4/4.1/4.2 and SSSE3 features to compilation

### 2020-03-03

#### Changed

- Update the following dependencies to the versions listed:
    - bootloader: v0.8.8
    - goblin: v0.2.0
    - linked_list_allocator: v0.6.6
    - proc-macro2: v1.0.9
    - register: v0.4.2
    - syn: v1.0.16
    - uart_16550: v0.2.4
    - x86_64: v0.9.5

#### Removed

- Remove the following dependencies from the cargo dependency tree: array-init, cast, nodrop, rustc_version, semver, semver-parser, ux
