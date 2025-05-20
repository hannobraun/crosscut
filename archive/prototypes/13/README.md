# Crosscut - Prototype 13

## Context

This is one of many prototypes of the Crosscut programming language and
development environment. Check out the top-level README in this repository for
more information.

I started working on this prototype on 2025-01-26, following
[prototype 12](../12/), which had followed the long-running
[prototype 11](../11/). After a few weeks of development on the new prototype, I
decided to revert one of the decisions I had made

Specifically, I switched from prefix syntax back to postfix syntax (which
previous prototypes had been based on). But this time based on a new evaluation
model, more closely inspired by functional programming.

This prototype was not a direct continuation of the previous one, though heavily
based on it, with much code copied over.

## Outcome

The switch back to postfix syntax, while it seemed to make sense at the time,
eventually turned out to be ill-fated.

Using postfix syntax with an evaluation model closely inspired by functional
languages, put me in no man's land design-wise, with no other languages to look
at for inspiration.

Following a more general change in my thinking, towards focussing my innovation
on the core interactive features, and deciding to go with more traditional
approaches in other areas, I decided that this innovative approach to syntax was
a liability rather than an asset. Prefix syntax, based on the same functional
evaluation model, would be more manageable.

At the same time, another insight developed, regarding the rather free-form
approach I had taken towards structural code editing (which the postfix syntax
had also been clashing with). I came to the conclusion that I wanted to change
to a _more_ structured approach to code editing.

Either of these changes would have justified archiving the current state as a
snapshot, so I did just that on 2025-04-21, and started working on the next
prototype (a direct continuation, based on the same code).

## Further Reading

I announced that I'm archiving this prototype in my daily note on
[2025-04-23](https://www.crosscut.cc/daily/2025-04-23) and mentioned it again on
[2025-04-30](https://www.crosscut.cc/daily/2025-04-30).

I wrote about the switch to restricted structural structural editing from
[2025-05-01](https://www.crosscut.cc/daily/2025-05-01) to
[2025-05-05](https://www.crosscut.cc/daily/2025-05-05), and posted an update on
the progress on [2025-05-14](https://www.crosscut.cc/daily/2025-05-14).

I started writing about the switch from postfix to prefix syntax on
[2025-05-06](https://www.crosscut.cc/daily/2025-05-06). As of this writing
(2025-05-14), that series is still ongoing.
