# Crosscut

Crosscut (formerly known as Caterpillar) was a project with the goal of creating
a **programming language** and development environment that reduces the feedback
loop between making a change and seeing it run as much as possible, creating an
**immediate connection** between developer and code.

The idea was to achieve this using **interactive programming**: The practice of
directly manipulating your running program, instead of going through an extended
loop of re-compiling, re-starting, then navigating to where you can test your
change.

The core premise of the project was the hypothesis, that interactive programming
is underutilized. That using it as the central element that the language and its
tooling are built around, can make development more fun, productive, and
intuitive.

## Status

From the start, this project had been organized as a series of prototypes. It
went through [many of those](archive/prototypes/) over the years, and I learned
a lot from each of them.

I still think that the latest approach (what's in the top-level directory of
this repository) showed promise, but it was also clear that much more work would
be required to get anywhere.

I eventually decided to end the project, not really because of anything about
the project itself, but because I came to the conclusion that working on these
huge projects was the wrong approach for me personally.

## Usage

Compile and start the development environment by executing `cargo run` in the
root directory of this repository. This requires a working
[Rust](https://rust-lang.org/) development setup.

## Inspiration

Crosscut drew inspiration from many sources. The following is an **incomplete**
list:

- [Inventing on Principle](https://vimeo.com/906418692)
- [Tomorrow Corporation Tech Demo](https://www.youtube.com/watch?v=72y2EC5fkcE)
- [Stop Writing Dead Programs](https://jackrusher.com/strange-loop-2022/)

## License

This project is open source, licensed under the terms of the
[Zero Clause BSD License] (0BSD, for short). This basically means you can do
anything with it, without any restrictions, but you can't hold the authors
liable for problems.

See [LICENSE.md] for full details.

[Zero Clause BSD License]: https://opensource.org/licenses/0BSD
[LICENSE.md]: LICENSE.md
