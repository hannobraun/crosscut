# Crosscut - Prototype 12

## About

This is one of many prototypes of the Crosscut programming language and
development environment. Check out the top-level README in this repository for
more information.

I started working on this prototype on 2025-01-04, following the long-running
[prototype 11](../11/). That prototype had developed a number of problems,
including a lack of focus on the interactive features that are core to
Crosscut's premise.

Some of the other problems were rooted in unnecessary complexity, so I decided
to go for a from-scratch restart with this new prototype, enabling a radical
simplification in many areas.

One of those simplifications was switching to a tried and true syntax and
evaluation model, going from postfix operators and stack-based evaluation, to
more traditional prefix operators and a model largely inspired by lambda
calculus.

I eventually changed my mind on that, and decided to go again with a less
traditional approach. This was again using postfix operators, but more closely
inspired by functional languages. So basically, a fusion of the two previous
concepts.

To implement this change, I started working on the next prototype on 2025-01-26.
The next prototype was based on this one, inheriting the general approach and
quite a bit of code, but was not a direct continuation.

I eventually decided to archive this prototype on 2025-02-06.

## Further Reading

I announced that I was working on this prototype in my daily thought on
[2025-01-08](https://www.crosscut.cc/daily/2025-01-08). This started a
weeks-long series on the issues with the previous prototype, and how I was going
to address them with this one.

That series concluded with the thought on
[2025-02-03](https://www.crosscut.cc/daily/2025-02-03). Of particular note is
the thought on [2025-01-10](https://www.crosscut.cc/daily/2025-01-10), which
features an overview of all those issues.

On [2025-02-04](https://www.crosscut.cc/daily/2025-02-04), I announced the next
prototype. This started a series on the syntax and evaluation model, and how the
one in the next prototype was going to be different. As of this writing
(2025-02-06), this series is still ongoing.
