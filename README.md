# Spawner

[![Travis CI](https://travis-ci.org/durka/spawner.svg)](https://travis-ci.org/durka/spawner)

A tiny crate providing a wrapper for `thread::spawn` that can optionally auto-join threads when it goes out of scoped.

It's the less useful half of `thread::scoped` (the more useful half being support for non-move closures).

