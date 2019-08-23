/// The IOF directory implements modules related to the kernel IO framework. The IOF borrows the io concept from go and applies this at the kernel level, ensuring that the entire operating system uses an easy to understand, easy to implement, easy to operate framework.

/// The io module contains traits and functions related to the IOF itself. Implementors of the IOF must use the traits defined in this module.
pub mod io;
