===description===
Negative control for the promoted-property docblock-literal-union fix: a
docblock scalar type from a genuinely DIFFERENT family than the native
hint (`@param bool` on a native `int` hint) still defers to the native
hint, same as the ordinary (non-promoted) `@param` scalar-family guard.
===config===
suppress=UnusedParam
===file===
<?php
final class Wrong {
    /** @param bool $value */
    public function __construct(public int $value) {}
}
function f(Wrong $w): string {
    return $w->value;
}
===expect===
InvalidReturnType@7:4-7:21: Return type 'int' is not compatible with declared 'string'
