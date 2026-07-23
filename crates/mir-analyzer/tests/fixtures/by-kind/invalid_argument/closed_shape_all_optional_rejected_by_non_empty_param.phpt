===description===
A closed shape whose only properties are optional (e.g. `array{a?: int}`)
can still be `[]` at runtime — a non-empty properties map alone doesn't
prove `non-empty-array`/`non-empty-list`, unlike a genuinely required
property or an open shape (which may hide an unknown non-empty key).
===config===
suppress=MissingParamType,UnusedParam
===file===
<?php
/** @param non-empty-array<string, int> $counts */
function takesNonEmptyArray(array $counts): void {}

/**
 * @param array{a?: int} $allOptional
 * @param array{a: int} $required
 * @param array{...} $open
 */
function test(array $allOptional, array $required, array $open): void {
    takesNonEmptyArray($allOptional);
    takesNonEmptyArray($required);
    takesNonEmptyArray($open);
}
===expect===
InvalidArgument@11:23-11:35: Argument $counts of takesNonEmptyArray() expects 'non-empty-array<string, int>', got 'array{'a'?: int}'
