===description===
FP: a template nested inside an intersection param alternative
(`(T&Countable)|null`) wasn't recognized as carrying the template, so passing
`null` (matched by the sibling `|null` alternative) bound `T` to the raw `null`
argument instead of being recognized as fully explained by that alternative —
producing a bogus InvalidTemplateParam against T's bound.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
interface Countable2 {}
class Unrelated {}

/**
 * @template T of Countable2
 * @param (T&Countable2)|null $value
 */
function maybeCount($value): void {}

maybeCount(null);
// Sanity control: the bound must still be enforced for a genuine violation —
// this fix must not disable bound-checking for the whole pattern.
maybeCount(new Unrelated());
===expect===
InvalidArgument@14:11-14:26: Argument $value of maybeCount() expects 'T&Countable2|null', got 'Unrelated'
InvalidTemplateParam@14:0-14:27: Template type 'T' inferred as 'Unrelated' does not satisfy bound 'Countable2'
