===description===
A `@var` type the native property type rules out is ignored in favour of the native type.
===file===
<?php
final class F {
    /** @var string|null */
    public int $n = 0;
}
function f(F $f): int {
    $f->n = 1;
    return $f->n;
}
