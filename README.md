# RUSTJVM

A toy JVM, recreationally written in Rust. :crab: 

## Motivation

I started this project with the goal of learning Rust while still doing something  ~~_useful_~~ interesting. The idea was to make it the smallest possible.

## About

This project is far far away from been a complete, full-fledged, coffee brewer JVM (or even a toy one?) but it still _works_ to run some very simple bytecode. It supports primitive data types and its operations, including 1-dimension arrays, floating point arithmetic.

If feeling yourself adventurous today, you can run it the in following way :

`cargo +nightly run`

The above will run `Example.java` (actually its compiled class file counterpart) which is just a classic recursive fibonacci implementation. 

If you run it, you'll notice it's no fast, but I personally think it's a nice take to anyone looking for a basic virtual machine implementation.

## Going forward

I don't have enough time to keep working on it, but if you're interested, feel free to use any code you see here however you like. I might dig in here from time to time.
