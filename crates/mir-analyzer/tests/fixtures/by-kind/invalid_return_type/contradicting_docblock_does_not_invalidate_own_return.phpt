===description===
FN: a @return docblock that contradicts the native hint (`@return string`
on `: int`) made the function's OWN `return 1;` look invalid — the stored
"declared" return type became the wrong docblock value with no fallback to
the native hint. The native hint is runtime truth: PHP guarantees this
function always returns an int regardless of what the docblock claims, so
`return 1;` is valid. MismatchingDocblockReturnType still separately flags
the contradiction itself.
===config===
suppress=MismatchingDocblockReturnType
===file===
<?php
/**
 * @return string
 */
function g(): int { return 1; }
===expect===
