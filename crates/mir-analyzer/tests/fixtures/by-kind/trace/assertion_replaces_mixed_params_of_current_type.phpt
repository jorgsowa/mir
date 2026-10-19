===description===
A `@psalm-assert` replaces a current type whose params are `mixed`, even
when another param (the key) is narrower: `array<string, mixed>` asserted
`array<string>` must not keep `mixed` values (webmozart `all*()` patterns).
A current type that is as precise as the assertion in every param keeps its own.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @psalm-assert array<string> $value */
function isStringArray(mixed $value): void {}

/** @psalm-assert list<string> $value */
function isStringList(mixed $value): void {}

/** @psalm-assert iterable<string> $value */
function isStringIterable(mixed $value): void {}

/**
 * @param array<string, mixed> $keyed
 * @param list<mixed> $list
 * @param array<string, int> $ints
 * @param array{a: mixed} $shape
 * @param array<string, string> $strict
 */
function f(array $keyed, array $list, array $ints, array $shape, array $strict): void {
    isStringArray($keyed);
    /** @trace $keyed */
    $a = 1;
//  ^^^^^^^ Trace: Type of $keyed is array<int|string, string>
    isStringList($list);
    /** @trace $list */
    $b = 1;
//  ^^^^^^^ Trace: Type of $list is list<string>
    isStringIterable($shape);
    /** @trace $shape */
    $c = 1;
//  ^^^^^^^ Trace: Type of $shape is iterable<int|string, string>
    isStringArray($strict);
    /** @trace $strict */
    $d = 1;
//  ^^^^^^^ Trace: Type of $strict is array<string, string>
}
