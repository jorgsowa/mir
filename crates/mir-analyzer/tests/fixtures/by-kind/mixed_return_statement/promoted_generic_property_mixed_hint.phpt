===description===
FP: a promoted constructor property whose native type hint is the generic
placeholder `mixed` should take its real type from the `@param T` docblock
(with `@template T` on the class), the same way a plain `@var T` property
already does. Previously the promoted-property collector only preferred the
docblock over the native hint for the analogous unspecialized-`array` case,
so a `mixed`-hinted generic property stayed `mixed` and every read of it
was flagged as a mixed access/return.
===file===
<?php
/**
 * @template T
 */
final class Wrapper {
    /** @param T $value */
    public function __construct(public readonly mixed $value) {}
}

/** @param Wrapper<int> $w */
function f(Wrapper $w): int {
    return $w->value + 1;
}
===expect===
