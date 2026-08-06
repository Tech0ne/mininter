<p align="center">
  <img src="images/readme/icon.png" alt="Mininter logo (AI generated)" width="300">
  <br>
  <em>Mininter logo (AI generated)</em>
</p>

# mininter

Rust MiniScript & MicroScript interpreter and semi compiler

## What is MiniScript

Please see [here](https://miniscript.org/) for a detailed history, explanation, and tutorials on what is MiniScript

## What is MicroScript

It is a personal "superset" to MiniScript, designed to be a sort of middle ground between the ease and "beginner friendly" aspects of MiniScript, and the less verbose syntax of other languages like C or Rust.

When using MiniScript, I found that many times I was like "using end for to close a loop seems too clunky for me, i'm used to braces !"

Thus, I made MicroScript, essentially MiniScript with braces (and some other **syntax** changes) (a big emphase on the **SYNTAX** changes: MiniScript and MicroScript both uses the **same environement** and are meant to **cohexist**)

That means you can load both MiniScript and MicroScript files with the same context.
