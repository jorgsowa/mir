===description===
MismatchingDocblockReturnType fires when the docblock @return contradicts the
native hint (wider or disjoint). Narrowing docblocks, template returns and
matching types stay silent.
===file===
<?php
/** @return string|null */
function widerThanHint(): string { return 'x'; }

/** @return int */
function disjointFromHint(): string { return 'x'; }

/** @return non-empty-string */
function narrowsHint(): string { return 'x'; }

/** @return string */
function exactMatch(): string { return 'x'; }

/**
 * @template T
 * @param T $x
 * @return T
 */
function templated(mixed $x): mixed { return $x; }
===expect===
MismatchingDocblockReturnType@3:10-3:23: Docblock return type 'string|null' does not match inferred 'string'
MismatchingDocblockReturnType@6:10-6:26: Docblock return type 'int' does not match inferred 'string'
InvalidReturnType@6:39-6:50: Return type '"x"' is not compatible with declared 'int'
