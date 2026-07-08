===description===
`static $x = <expr>;` should seed $x's type from the initializer expression
instead of unconditionally falling back to mixed.
===config===
suppress=UnusedVariable
===file===
<?php
function foo(): void {
    static $x = "hello";
    /** @mir-check $x is string */
    $_ = 1;
}
===expect===
