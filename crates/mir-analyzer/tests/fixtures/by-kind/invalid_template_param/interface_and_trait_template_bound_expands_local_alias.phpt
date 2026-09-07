===description===
An interface's or trait's own `@template T of Alias` bound never expanded
a same-file `@psalm-type` alias before resolution — `class.rs`'s
template-param construction already ran `expand_aliases_only` first, but
`interface.rs`/`trait.rs` resolved the raw docblock type directly (and
built their `type_aliases` map too late to even use it), so `IntOrFloat`
was namespace-qualified as if it were a real (nonexistent) class instead
of expanding to `int|float` — any implementer's bound check then ran
against that phantom class instead of the real bound.
===config===
suppress=UnusedVariable,MissingReturnType,MissingConstructor
===file===
<?php
/**
 * @psalm-type IntOrFloat = int|float
 * @template T of IntOrFloat
 */
interface Box {}

// Satisfies the (real) bound int|float — no error.
/** @implements Box<int> */
class IntBox implements Box {}

// Violates the bound — string is not int|float.
/** @implements Box<string> */
class StringBox implements Box {}

/**
 * @psalm-type IntOrFloat = int|float
 * @template T of IntOrFloat
 */
trait BoxTrait {}
===expect===
InvalidTemplateParam@14:0-14:33: Template type 'T' inferred as 'string' does not satisfy bound 'int|float'
