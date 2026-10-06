===description===
MismatchingDocblockReturnType fires when the docblock @return contradicts the
native hint (wider or disjoint). Narrowing docblocks, template returns and
matching types stay silent. Each function's own `return 'x';` must NOT also
trigger InvalidReturnType — the native hint is runtime truth regardless of
what a contradicting docblock claims.
===file===
<?php
/** @return string|null */
function widerThanHint(): string { return 'x'; }
//       ^^^^^^^^^^^^^ MismatchingDocblockReturnType: Docblock return type 'string|null' does not match inferred 'string'

/** @return int */
function disjointFromHint(): string { return 'x'; }
//       ^^^^^^^^^^^^^^^^ MismatchingDocblockReturnType: Docblock return type 'int' does not match inferred 'string'

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
