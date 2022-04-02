//! This crate provides useful derive macros for various `magma` functionality

#![recursion_limit = "128"]

use proc_macro::TokenStream;

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
