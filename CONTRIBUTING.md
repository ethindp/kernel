# Contributing to Kernel

Welcome and thank you for wanting to contribute to this little kernel of mine! Though this kernel was initially started as an educational desire for me to learn OS development, I am hoping that it will get somewhere and will be able to run applications and support a full userspace environment.

This project isn't designed to be competing with the mainstream OSes out there (that would be a nightmare and would be highly impractical). In order of priority, the goals of this project are:

1. To build a fast, stable, secure operating system for general-purpose use, following the semantic versioning guidelines, but willing to break things when needed;
2. To write code that is readable and easy to maintain and learn from to help people get onboard with the project and to learn about OS development, hardware, and OS design;
3. To take advantage of the latest hardware and technological advancements, and to not persue backwards compatibility unless absolutely necessary; and
4. To have fun.

With that in mind, there are some rules to follow when contributing to the project.

## Contribution guidelines

### Licensing

Its perfectly reasonable to reuse existing code out there. However, as with all open-source projects, always, always ensure that the code you want to incorporate does not violate the license (currently MPL-2.0). If you have a question about licensing, don't hesitate to ask and we'll investigate it together.

Similarly, if I've made licensing mistakes, don't hesitate to call me out on it. The dependency graph of this project is quite complex, and though I've done my best to ensure that all the licenses are compatible, I can't always succeed at that (since I don't have absolute control over all the dependencies of the project, unfortunately).

### Code quality

This project enables nearly every Linter rust supports, and all of Clippy's linters. Additionally, linting is done as a part of the build process. The reasoning behind this is twofold:

* It enforces good code quality and hygiene and readability; and
* It eliminates warnings as much as possible.

Though I'm willing to agree on changing the linters that the project uses, please ensure you describe why a linter shouldn't or should be used, if an explanation is not already available within the Rust documentation (e.g. an edge-case where the Linter fails). If, however, the documentation is sufficient enough to explain the linters usage and rationale, you can simply point people to that.

As noted above, we also use all of Clippy's linters.

If you feel a linter should be set to `deny` instead of `forbid`, please provide a good explanation as to why (e.g. for async functionality, structs needed to be declared that did not support `Copy`/`Debug`). The rationale for this is that we want to have strict enforcement of Rust language and borrow checker semantics, and altering a linter level from `forbid` to `deny` opens up a potential area where that linter could be bypassed. Though optimization (should) take care of most of the issues that this may introduce (e.g. dead code elimination), its much easier to simply disallow that linter entirely, in the case of the `dead-code` lint. This also makes debugging easier.

### Use of floating-point operations and SIMD

#### X86-specific

Please avoid the use of floating-point if at all possible. For every floating-point operation, an equivalent is achievable using fixed-point arithmetic. Though Rust provides a software floating-poitn arithmetic library via the `soft-float` target feature, please avoid its use, as software FP is slow. Additionally, avoid FP because the kernel is not set up to handle FP errors, either from the x87 FPU or via SIMD.

The ultimate rule is: if your using floating-point operations, your doing it wrong. All hardware uses integers; your not going to read a floating-point value from an NVMe controller or PIT/APIC, and converting a value to FP just to make the math simpler to understand, just to convert it back to an integer to write it back to a hardware register, is a waste of time and CPU cycles, and could potentially cause a crash if you happen to cause an FP exception (e.g. division by zero). If your wanting to use FP, re-think how you want to implement whatever it is that your doing, and use integer math whenever possible, even if that involves bitmasks and bit shifts.

Similarly, avoid SIMD unless its an absolute necessity (e.g. cryptography). SIMD/AVX needs to be explicitly enabled by the kernel, and requires the kernel to save and restore register states every context switch (including in interrupts). This is a costly operation, particularly when switching privilege rings, and so this should be minimized.

#### Other architectures

The use of FP/SIMD on other architectures is dependent on the architecture in question. Generally, unless otherwise indicated by the architecture ISA documentation, the guidelines are thus:

* Avoid SIMD/FP unless there is a legitimate gain in kernel space. The kernel should not need FP/SIMD. If your doing something like encryption, use an algorithm like ChaCha20/XChaCha20 (which is designed for use in software) unless you *must* use AES.
* If an operation can be expressed without the use of SIMD, go with the more expressive version, instead of the fastest. This is because SIMD can be difficult to understand, even if someone is well-versed in the subject, and some architectures (like x86) place specific requirements on SIMD usage (in particular, x86 requires a 16-byte alignment).
* If an architecture requires the kernel to save register context for SIMD, avoid SIMD. Userspace software is going to be requiring us to save enough register contexts; we don't need extra complexity and saving requirements in kernel space as well.

### Documentation

Document everything. Even a one-line summary is enough as long as you explain what the function your implementing does. The more, the better, but always document the functions you export. (Rust will force you to do this, but even internal functions should be documented if the name of the function doesn't make it obvious what the function is supposed to do or if the implementation of the function is complex and its difficult to determine what it does from reading its code).

### Intrinsic functions; inline assembly

If you need to use an intrinsic function or the `asm!` macro, generalize it. For example, if you need to use the [HLT](https://www.felixcloutier.com/x86/hlt) instruction, generalize it to a more architecture-neutral interrupt-waiting routine, like so:

```rust
#[inline]
pub fn halt() {
    if cfg!(any(target_arch="x86", target_arch="x86_64")) {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    } else if ... {
        // More architectures here...
    }
}
```

Notice how this function is not declared as `unsafe`. This is because not all of the function (needs) to be unsafe, and it, in fact, does not do anything unsafe at all. Rust will also enforce this paradigm via the `unsafe_op_in_unsafe_fn` linter.

The reasoning is because operations like this are general operations that apply to every architecture known today. This will, additionally, make it much easier to port the kernel to other architectures when the time comes.

## Project structure

The kernel consists of two separate crates that are combined to make a final executable during the build process: `libk`, the kernel library, which does all the heavy lifting, and the kernel binary `kernel`, which provides a minimal executionstub that brings the system up into a sane state for `libk` to work in. After the main kernel binary completes its initialization routine (which includes setting up interrupt handlers/controllers, descriptor tables, basic platform configuration, etc.), it calls `libk::init()`, which takes control of the system and brings it up into a fully operational state. This separation makes it possible to swap out the kernel library used, which makes this kernel quite flexible.

The full initialization routine is explained in the documentation, located in the docs directory.

In general, modifying the kernel binary itself is rarely necessary (though sometimes desirable). 99 percent of the time, any and all modifications and additions should be done through `libk`.

The directory layout is as follows:

* `docs`: contains kernel/libk documentation
* `drivers`: contains kernel drivers (these will be explained below)
* `libk`: The kernel library
* `src`: the kernel minimal execution stub binary

#### Note

The `drivers` directory does not yet exist because issue #33 is preventing driver development, and not all the peaces are in place yet. Issue #14 describes the requirements that this project is aiming for in more detail.

### `docs` directory

The `docs` directory contains kernel documentation. This documentation should be as extensive as possible. Though `rustdoc` is used to generate code documentation, driver documentation should be stored here. Kernel documentation is written using markdown and the `mdbook` tool.

The documentation is structured similarly to the main repository structure.

### `drivers` directory

The `drivers` directory is where kernel drivers will be stored. The kernel driver interface is unspecified at this time. For now, this directory doesn't exist. Once we have figured out the kernel driver interface, we can see about drivers.

### `libk` directory

This is the kernel library. This is where all the main kernel code resides. The source codestructure is as follows:

* `arch`: In future, this directory will be used to store architecture-specific code (e.g. paging structures, control register manipulation functions, etc.)
* `task`: this directory contains the multitasking code

The remaining files in this directory are modules for kernel features that need to be in the kernel for the platform to be reasonably functional (that is, for the kernel to be able to execute code (processes, threads, multiprocessing) and to manage the platform (processor, RAM, power management, etc.)).

## Contributor workflow

If you've gotten this far, you clearly wish to contribute something (or many things). Thank you -- your work and assistance are greatly appreciated and we're glad to have you on board!

If your not a contributor of the repository, the contribution workflow is as follows. Please do your best to follow this workflow closely.

1. Fork the repository (`gh repo fork`). Add the "upstream" remote and set the default "origin" remote to the fork when prompted. This will ensure that you can pull down changes to upstream to keep your repository in sync. After this step is complete, you can avoid doing this in future.
2. Push the repository so that GitHub is notified of the remote changes that `gh` made to the fork.
3. Make any changes you desire. Please keep your changes small -- you can submit multiple commits, but try to avoid submitting large swaths of code changes in a single commit; this makes code review difficult and it takes longer to complete. Additionally, please ensure that all of your code builds and passes all linter checks. It doesn't need to work as intended; however, it needs to at least build. (Once we get GitHub actions going, this should be done as a part of the PR process, and you can skip this part and simply write code.)
4. Commit and push your changes.
5. Open a pull request against this repository, not your fork.

If your a contributor to this repository (that is, you have contributor access), the workflow is as follows:

1. Clone this repository. Do not fork it.
2. Make any changes you desire. All the rules for step 3 above apply.
3. Commit and push your changes.


