===description===
Regression guard for the nullable-hint-vs-docblock check: when the
docblock already reflects nullability (either via `?T` shorthand or a
`T|null` union), or the hint isn't nullable at all, there's no
contradiction and nothing should be flagged.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param ?string $a
 */
function viaQuestionMark(?string $a): void {}

/**
 * @param string|null $b
 */
function viaUnion(?string $b): void {}

/**
 * @param string $c
 */
function nonNullableHint(string $c): void {}
===expect===
