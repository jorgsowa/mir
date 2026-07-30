===description===
Regression control for the promoted-property docblock-literal-union fix:
the previously-supported "plain `mixed`/`array` native hint + `@param T`
docblock" idiom (a class-level template with no way to natively express
the template param) must still attach the docblock template type — the
fix generalizes the old special case rather than replacing it.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @template T */
final class Box {
    /** @param T $value */
    public function __construct(public mixed $value) {}
}
function f(Box $box): int {
    /** @var Box<int> $intBox */
    $intBox = $box;
    return $intBox->value + 1;
}
===expect===
