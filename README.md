# Crosscut

Crosscut (formerly known as Caterpillar) is a **programming language** and
development environment with a dual goal:

- Reduce the feedback loop between making a change and seeing it run as much as
  possible, creating an **immediate connection** between developer and code.
- Bring this experience to many places: browsers, servers, desktops, phones,
  watches, microcontrollers; CPUs and GPUs.

We aim to achieve this using **interactive programming**: The practice of
directly manipulating your running program, instead of going through an extended
loop of re-compiling, re-starting, then navigating to where you can test your
change.

The core premise of this project is the hypothesis, that interactive programming
is underutilized. That using it as the central element that the language and its
tooling are built around, can make development more fun, productive, and
intuitive.

## Status

There's a very basic language, and a [very basic game][Snake] implemented in it.
So far, not much thought has been given to using Crosscut outside of this
specific scenario. If you want to experiment with it, your best bet is to clone
this repository and play around with what's already here (see _Usage_ below).

Interactive programming is currently being implemented (it was already present
in a [previous prototype](archive/prototypes/07/)). Right now, the most basic
cases work, but most changes will result in undefined behavior. Fixing this is
an ongoing process.

The other part of the goal, bringing the interactive development experience to
many places, is _not_ being worked on yet. Right now, the focus is to create a
basic solution for a single use case (game development) on a single platform
(browsers).

You can keep up with the project by reading my [daily thoughts], which include
development updates.

## Usage

As per _Status_ above, not much thought has been given to using Crosscut outside
of this repository. If you want to play around with it, playing around with this
repository is the way to do it:

1. Clone this repository.
2. Run `cargo run` (requires [Rust](https://rust-lang.org/) to be installed).

That second step will start a build tool that should compile everything in here,
start a development server, and eventually output a link.

Open that link in your browser. You should see the Snake game running, alongside
the Crosscut debugger.

You can modify the [code of the game][Snake] or any of the
[Crosscut code](capi/) itself. The builder should automatically re-compile every
change, and the web page should reload, if necessary.

Please note that there currently is a
[substantial number of bugs and other basic problems](https://github.com/hannobraun/crosscut/issues).

## Inspiration

Crosscut draws inspiration from many sources. The following is an **incomplete**
list:

- [Inventing on Principle](https://vimeo.com/906418692)
- [Tomorrow Corporation Tech Demo](https://www.youtube.com/watch?v=72y2EC5fkcE)
- [Stop Writing Dead Programs](https://jackrusher.com/strange-loop-2022/)

## Design

This section aims to document the decisions that have gone into the language
design. Due to the current state of the project, this isn't (yet) a long list,
nor is anything here going to be final.

For more information on future design directions, please follow my
[daily thoughts].

### Developer experience is the priority, but not an absolute one

Crosscut's core premise is to improve developer experience using interactive
programming. But this does not make developer experience an absolute priority.
It must always be weighed against other factors.

Where developer experience conflicts with performance, the decision will be made
configurable, if practical. Where possible, designs are chosen that are good for
both.

And while a focus on developer experience might decrease _peak_ performance, it
might improve performance in general. Simply because developers have the tools
and spare resources to make their programs run fast.

The following daily thoughts provide more context:
[2024-08-30](https://capi.hannobraun.com/daily/2024-08-30),
[2024-09-01](https://capi.hannobraun.com/daily/2024-09-01),
[2024-09-02](https://capi.hannobraun.com/daily/2024-09-02),
[2024-09-03](https://capi.hannobraun.com/daily/2024-09-03), and
[2024-09-04](https://capi.hannobraun.com/daily/2024-09-04).

### Experimentation first; conservative decisions later, as necessary

I want Crosscut to be adopted. That could mean that I need to focus innovation
where that provides the most benefit, and keep other aspects of the language
conservative and familiar.

Before that becomes necessary, I want to experiment though. At least give the
language to chance to be more than an incremental improvement over the status
quo.

The following daily thoughts provide more context:
[2024-06-18](https://capi.hannobraun.com/daily/2024-06-18) and
[2024-06-19](https://capi.hannobraun.com/daily/2024-06-19).

### Continued evolution over backwards compatibility

I'm not targeting a 1.0 release after which the language is expected to have few
or no breaking changes. Right now, everything is early-stage and experimental
anyway. But even long-term, I don't want to commit to backwards compatibility.
The continued evolution of the language and the costs of ongoing maintenance
will be prioritized instead.

As the language matures, there will be a growing focus on making any given
upgrade easy. But each release might introduce changes that require updates to
Crosscut code. Where possible, users will be given ample time to make those
changes, or they will be automated outright.

The following daily thoughts provide more context:
[2024-05-28](https://capi.hannobraun.com/daily/2024-05-28),
[2024-05-29](https://capi.hannobraun.com/daily/2024-05-29),
[2024-05-31](https://capi.hannobraun.com/daily/2024-05-31),
[2024-06-01](https://capi.hannobraun.com/daily/2024-06-01),
[2024-06-02](https://capi.hannobraun.com/daily/2024-06-02),
[2024-06-03](https://capi.hannobraun.com/daily/2024-06-03), and
[2024-06-05](https://capi.hannobraun.com/daily/2024-06-05).

### Simplicity over familiarity and short-term convenience

Crosscut strives to be simple, even where that is unfamiliar to most developers
(because it's different to other languages), and even where that might be
inconvenient in some situations.

Simplicity makes the language easier to understand. For the developer, but also
for tooling, which can then provide a better experience to the developer. Unless
this advantage is clearly outweighed by other factors, the simple design should
be chosen.

An example of this is not supporting early returns. Functions in Crosscut (or
rather their branches, to be precise) have a single exit point, which can make
code easier to follow for the developer, but specifically allows the debugger to
do call stack reconstruction.

The following daily thoughts provide more context:
[2024-08-25](https://capi.hannobraun.com/daily/2024-08-25) and
[2024-08-26](https://capi.hannobraun.com/daily/2024-08-26).

### Minimalism over convenience, for now

For the time being, to make iterating on the language easier, simplicity trumps
convenience. This means that a small number of concepts is used to cover a wide
range of use cases.

To give a concrete examples, pattern-matching functions are the single tool used
to represent all control flow (both conditionals, via the pattern matching; and
iteration, via recursion).

This is sometimes more verbose than more specific constructs, which makes the
language less convenient to write and harder to read. For now, I consider this
to be an acceptable trade-off.

Later on, it will make more and more sense to add syntax sugar, thereby
increasing convenience, while under the hood still mapping the new syntax to
simple primitives.

The following daily thoughts provide more context:
[2024-07-24](https://capi.hannobraun.com/daily/2024-07-24),
[2024-08-08](https://capi.hannobraun.com/daily/2024-08-08),
[2024-08-10](https://capi.hannobraun.com/daily/2024-08-10),
[2024-08-12](https://capi.hannobraun.com/daily/2024-08-12),
[2024-08-13](https://capi.hannobraun.com/daily/2024-08-13),
[2024-08-14](https://capi.hannobraun.com/daily/2024-08-14),
[2024-08-15](https://capi.hannobraun.com/daily/2024-08-15),
[2024-08-16](https://capi.hannobraun.com/daily/2024-08-16),
[2024-08-17](https://capi.hannobraun.com/daily/2024-08-17),
[2024-08-18](https://capi.hannobraun.com/daily/2024-08-18),
[2024-08-20](https://capi.hannobraun.com/daily/2024-08-20),
[2024-08-21](https://capi.hannobraun.com/daily/2024-08-21), and
[2024-08-22](https://capi.hannobraun.com/daily/2024-08-22).

### Untyped now, statically typed later

Crosscut is currently untyped. This means there is only a single data type,
32-bit words, and all values are represented using those. The developer often
has to choose a specific operation, depending on what specific type they expect
(`add_s8` vs `add_s32`, for example).

This isn't a final decision. It just was the easiest place to start in. The goal
is to make the language statically typed. I expect this to be a gradual process,
where the compiler understands more and more about types over time, until it can
select the correct operation itself (and the developer can just call `add` or
`+`).

The following daily thoughts provide more context:
[2024-07-16](https://capi.hannobraun.com/daily/2024-07-16),
[2024-07-17](https://capi.hannobraun.com/daily/2024-07-17), and
[2024-08-23](https://capi.hannobraun.com/daily/2024-08-23).

### Crosscut code is embedded into a host

Crosscut code does not execute I/O operations directly. Whenever it needs to do
something that affects the outside world, it triggers an effect. That effect is
executed by a platform-specific piece of code called the "host", which then
passes the result back to Crosscut.

At this point, this is just an implementation detail, but I hope to use this
concept to realize a number of benefits in the future: Mainly to represent all
I/O resources as values provided by the host, making Crosscut code sandboxed by
default, and allowing side effects to be tracked through those values, making
all functions pure.

A lot of this is inspired by the "platform" concept in [Roc].

The following daily thoughts provide more context:
[2024-07-02](https://capi.hannobraun.com/daily/2024-07-02),
[2024-07-03](https://capi.hannobraun.com/daily/2024-07-03),
[2024-07-04](https://capi.hannobraun.com/daily/2024-07-04),
[2024-07-05](https://capi.hannobraun.com/daily/2024-07-05),
[2024-07-06](https://capi.hannobraun.com/daily/2024-07-06),
[2024-07-07](https://capi.hannobraun.com/daily/2024-07-07)

[Roc]: https://www.roc-lang.org/

### Postfix operators

The language uses postfix operators, like `arg1 arg2 do_thing` or `1 2 +`, as
opposed to prefix (`do_thing(arg1, arg2)`, `(+ 1 2)`) or infix (`1 + 2`)
operators.

To keep the language simple, I want to (at least initially) restrict it to one
type of operator. I believe postfix operators are the best option under that
constraint, due to their combination of simplicity, conciseness, and natural
support for chaining operations. That comes at the cost of familiarity.

The following daily thoughts provide more context:
[2024-05-03](https://capi.hannobraun.com/daily/2024-05-03),
[2024-05-04](https://capi.hannobraun.com/daily/2024-05-04),
[2024-05-05](https://capi.hannobraun.com/daily/2024-05-05),
[2024-05-06](https://capi.hannobraun.com/daily/2024-05-06),
[2024-05-07](https://capi.hannobraun.com/daily/2024-05-07),
[2024-05-08](https://capi.hannobraun.com/daily/2024-05-08),
[2024-05-09](https://capi.hannobraun.com/daily/2024-05-09),
[2024-05-10](https://capi.hannobraun.com/daily/2024-05-10), and
[2024-05-11](https://capi.hannobraun.com/daily/2024-05-11).

### Stack-based evaluation, but not a stack-based language

Crosscut, featuring postfix operators, has a stack-based evaluation model. But
it is not a stack-based language. There is no single data stack that is used to
pass arguments and return values between functions.

Instead, Crosscut uses a much more conventional model, with a regular stack and
explicit function arguments. Each operand stack is local to a (function) scope.

This approach is less error-prone, but also less flexible and more verbose. It
seems to make sense right now, but as the language grows other features that
make it less error-prone (like static typing and better tooling), this decision
can be revisited.

The following daily thoughts provide more context:
[2024-05-10](https://capi.hannobraun.com/daily/2024-05-10),
[2024-05-11](https://capi.hannobraun.com/daily/2024-05-11),
[2024-06-20](https://capi.hannobraun.com/daily/2024-06-20),
[2024-06-21](https://capi.hannobraun.com/daily/2024-06-21),
[2024-06-22](https://capi.hannobraun.com/daily/2024-06-22),
[2024-06-23](https://capi.hannobraun.com/daily/2024-06-23),
[2024-06-24](https://capi.hannobraun.com/daily/2024-06-24), and
[2024-06-25](https://capi.hannobraun.com/daily/2024-06-25).

### Designed to be used with tooling

Crosscut is designed to be used with tooling and makes various trade-off that
benefit this intended use case, at the cost of other cases where tooling is not
available.

The following daily thoughts provide more context:
[2024-07-21](https://capi.hannobraun.com/daily/2024-07-21),
[2024-07-22](https://capi.hannobraun.com/daily/2024-07-22), and
[2024-07-23](https://capi.hannobraun.com/daily/2024-07-23).

### Compilation doesn't stop on errors

The compiler doesn't stop, when encountering an error. Instead it encodes the
error into the representation it is currently generating, still creating an
executable result in the end. If code generated from one of the build errors is
hit, this results in a runtime panic.

This has advantages, like allowing you to run tests or debug a piece of code,
even while something else doesn't currently typecheck, for example. This could
make day-to-day development easier. It also makes sure that the enriched version
of the source code that the debugger displays, is always available.

The following daily thoughts provide more context:
[2024-09-05](https://capi.hannobraun.com/daily/2024-09-05) and
[2024-09-06](https://capi.hannobraun.com/daily/2024-09-06).

## License

This project is open source, licensed under the terms of the
[Zero Clause BSD License] (0BSD, for short). This basically means you can do
anything with it, without any restrictions, but you can't hold the authors
liable for problems.

See [LICENSE.md] for full details.

[Snake]: games/snake/snake.capi
[daily thoughts]: https://capi.hannobraun.com/daily
[Zero Clause BSD License]: https://opensource.org/licenses/0BSD
[LICENSE.md]: LICENSE.md
