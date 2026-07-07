===description===
@psalm-pure must be recognized as an alias of @pure, the same way @psalm-
template/@phpstan-template already alias @template. ImpureFunctionCall must
not fire when a @pure function calls a @psalm-pure-only function.
===file===
<?php
/** @psalm-pure */
function double(int $n): int {
    return $n * 2;
}

/** @pure */
function quadruple(int $n): int {
    return double(double($n));
}
===expect===
