# System Calls

Issuing a system call requires three things, unlike any other operating system:

* A system call group (SCG) (also known as a system call category (SCC)): specifies the category of the system call that you'd like to execute. This is stored in the RAX register.
* System call code (stored in RBX, must be a system call within the given category)
* Parameters for the system call (stored in RCX, RDX, RSI, RDI, and R8-R15, allowing for a system call to have up to 12 parameters)

System calls are only usable via interrupts at this time. The interrupt to use for system calls is the same for linux -- 0x80.

## List of system calls

### Category 0 - Memory

| Name | Code (RBX) | Parameters |
| ---- | ---------- | ---------- |
| allocate_paged_range | 0 | start and end address of range |
