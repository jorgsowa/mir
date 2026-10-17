===description===
A `@psalm-assert` only narrows: a value already narrower than the asserted
type keeps its own type (webmozart/assert's `isIterable()` turned an
`array<class-string>` param into `iterable`). A wider value still takes the
asserted type.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @psalm-assert iterable $value */
function isIterable(mixed $value): void {}

/** @psalm-assert array<string> $value */
function isStringArray(mixed $value): void {}

/** @param array<string> $narrow */
function f(array $narrow, mixed $wide, array $plain): void {
    isIterable($narrow);
    /** @trace $narrow */
    $a = 1;
//  ^^^^^^^ Trace: Type of $narrow is array<int|string, string>
    isIterable($wide);
    /** @trace $wide */
    $b = 1;
//  ^^^^^^^ Trace: Type of $wide is iterable
    isStringArray($plain);
    /** @trace $plain */
    $c = 1;
//  ^^^^^^^ Trace: Type of $plain is array<int|string, string>
}
