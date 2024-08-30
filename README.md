# Caterpillar

Caterpillar is a **programming language** with a dual goal:

- Create an **immediate connection** between you and the code you're working on,
  using **interactive programming**.
- Bring this experience to many places: browsers, servers, desktops, phones,
  watches, microcontrollers; CPUs and GPUs.

Interactive programming is the practice of directly manipulating your running
program, instead of going through an extended loop of re-compiling, re-starting,
then navigating to where you can test your change.

The core premise of the project is the hypothesis, that interactive programming
is underutilized. That using it as the central element the language and its
tooling are built around, can make development more fun, productive, and
intuitive.

## Status

Caterpillar is still early-stage and experimental. It can hardly be called a
programming language right now. Development is focused on creating a basic
solution for one use case (game development) on one platform (browsers).

You can keep up with the project by reading my [daily thoughts], which include
development updates.

## Inspiration

Caterpillar draws inspiration from many sources. The following is an
**incomplete** list:

- [Inventing on Principle](https://vimeo.com/906418692)
- [Tomorrow Corporation Tech Demo](https://www.youtube.com/watch?v=72y2EC5fkcE)
- [Stop Writing Dead Programs](https://jackrusher.com/strange-loop-2022/)

## Design

This section aims to document the decisions that have gone into the language
design. Due to the current state of the project, this isn't (yet) a long list,
nor is anything here going to be final.

For more information on future design directions, please follow my
[daily thoughts]. There's also a [design document](design.md), which I'd like to
phase out, but that still provides some value.

### Experimentation first; conservative decisions later, as necessary

I want Caterpillar to be adopted. That could mean that I need to focus
innovation where that provides the most benefit, and keep other aspects of the
language conservative and familiar.

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
Caterpillar code. Where possible, users will be given ample time to make those
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

Caterpillar strives to be simple, even where that is unfamiliar to most
developers (because it's different to other languages), and even where that
might be inconvenient in some situations.

Simplicity makes the language easier to understand. For the developer, but also
for tooling, which can then provide a better experience to the developer. Unless
this advantage is clearly outweighed by other factors, the simple design should
be chosen.

An example of this is not supporting early returns. Functions in Caterpillar (or
rather their branches, to be precise) have a single exit point, which can make
code easier to follow for the developer, but especially allows the debugger to
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

Caterpillar is currently untyped. This means there is only a single data type,
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

### Caterpillar code is embedded into a host

Caterpillar code does not execute I/O operations directly. Whenever it needs to
do something that affects the outside world, it triggers an effect. That effect
is executed by a platform-specific piece of code called the "host", which then
passes the result back to Caterpillar.

At this point, this is just an implementation detail, but I hope to use this
concept to realize a number of benefits in the future: Mainly to represent all
I/O resources as values provided by the host, making Caterpillar code sandboxed
by default, and allowing side effects to be tracked through those values, making
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

Caterpillar, featuring postfix operators, has a stack-based evaluation model.
But it is not a stack-based language. There is no single data stack that is used
to pass arguments and return values between functions.

Instead, Caterpillar uses a much more conventional model, with a regular stack
and explicit function arguments. Each operand stack is local to a (function)
scope.

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

Caterpillar is designed to be used with tooling and makes various trade-off that
benefit this intended use case, at the cost of other cases where tooling is not
available.

The following daily thoughts provide more context:
[2024-07-21](https://capi.hannobraun.com/daily/2024-07-21),
[2024-07-22](https://capi.hannobraun.com/daily/2024-07-22), and
[2024-07-23](https://capi.hannobraun.com/daily/2024-07-23).

## Acknowledgements

I'd like to thank [Martin Dederer](https://github.com/martindederer) for
suggesting the name!

## License

This project is open source, licensed under the terms of the
[Zero Clause BSD License] (0BSD, for short). This basically means you can do
anything with it, without any restrictions, but you can't hold the authors
liable for problems.

See [LICENSE.md] for full details.

[daily thoughts]: https://capi.hannobraun.com/daily
[Zero Clause BSD License]: https://opensource.org/licenses/0BSD
[LICENSE.md]: LICENSE.md
