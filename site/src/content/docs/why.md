---
title: Why mir?
description: What mir is good at and how it differs from other PHP analyzers.
---

mir is a static analyzer for PHP written in Rust. It reads your code without
running it and reports bugs: wrong types, calls to things that don't exist,
null dereferences, unreachable branches, unused code and unsafe data flows.
This page explains what it's designed for.

## Fast on the first run, faster after that

mir analyzes files in parallel on every CPU core. On an Apple M1 laptop it checks the Symfony source tree (about 4,400 files) in roughly
two seconds from a cold start.

Results are cached per file by content hash in `.mir/cache`. On the next run
only files whose content changed are analyzed again, plus the files that depend
on a changed *declaration*. Editing a method body doesn't invalidate its
callers; changing its signature does. A re-run with no changes on the same
Symfony tree takes about half a second.

That speed changes how you use the tool: run it on every commit hook, in every
CI job and on every save, not once a night.

## A single binary

mir is one native executable. It doesn't need a PHP runtime to analyze PHP, so
the PHP version installed on the machine doesn't limit the code it can check.
Set `--php-version` (or `<phpVersion>`) anywhere from 7.4 to 8.6 and mir parses
your code with that version's grammar and loads only the built-in functions
and classes that exist in it.

The Composer package downloads the prebuilt binary for your platform and
verifies its checksum, so PHP projects can add it like any other dev
dependency. See [Getting Started](/mir/guides/getting-started/).

## Types that match how PHP is written

mir's type system covers what real PHP codebases annotate: generics and
templates with variance, array shapes and lists, literal types, `class-string<T>`,
`key-of` / `value-of`, conditional return types, intersection types,
assertions (`@assert`, `@assert-if-true`), `@param-out` and `@self-out`.

Types are narrowed by `if`, `match`, `switch`, `instanceof`, `is_*()`,
`isset()`, `empty()`, early returns and assertion functions, so a value checked
for `null` on one line isn't reported as possibly `null` on the next.

See [Docblock Annotations](/mir/reference/docblock/) for the full list.

## Security checks built in

Taint analysis runs by default. mir follows request data (`$_GET`, `$_POST`,
cookies, uploaded files, `php://input`, and the request objects of Symfony,
Laravel and PSR-7) through assignments, function calls and method calls, and
reports it when it reaches a dangerous sink: HTML output, SQL, shell commands,
file paths, `include`, `eval`, `unserialize`, HTTP headers, cookies, outgoing
requests (SSRF), LDAP, XPath, callables and LLM prompts.

Each report shows the path the data took:

```text
src/search.php:7:4 error[MIR0801] TaintedHtml: Tainted HTML output — possible XSS
  = taint flow: $_GET['q'] (line 4) → $q (line 4) → trim($q) (line 5) → $label (line 5)
```

Function bodies are summarized automatically, so you don't need to annotate
every helper for the analysis to see through it.

## Errors PHP would raise

Some code parses but PHP still refuses to compile it: a non-static
`__callStatic()`, a duplicate property, a property declared in an interface
without hooks. mir reports these as `CompileError` with PHP's
own wording for the target version, so you find them before the code is loaded
in production.

## Stable, documented diagnostics

Every issue has a stable code such as `MIR0201`. A code is never reused for a
different issue, so baselines, suppressions and CI filters keep working across
upgrades. Each code has a [reference page](/mir/reference/issues/) with an
example and a fix.

Output is available as plain text, JSON, GitHub Actions annotations, JUnit XML
and SARIF for code-scanning dashboards.

## Adopt it on an existing codebase

You don't have to fix every issue before mir is useful:

- A [baseline](/mir/guides/configuration/#baselines) records today's issues so
  only new ones are reported.
- [Inline suppressions](/mir/reference/suppressions/) silence individual false
  positives, and `--show-info` lists the ones that no longer match anything.
- Existing `@psalm-*` and `@phpstan-*` annotations are read as they are, so
  annotated code doesn't need rewriting. Psalm configuration, baselines and
  plugins are supported too; see [Migrating to mir](/mir/guides/migrating/).

## Built to be embedded

The analyzer is a set of Rust crates rather than a monolith. Its incremental
engine is built on [salsa](https://github.com/salsa-rs/salsa): a long-lived
session re-analyzes only the edited file and what depends on it, which is what
an editor integration needs. The same engine compiles to WebAssembly and runs the
[Playground](/mir/playground/) entirely in your browser.
