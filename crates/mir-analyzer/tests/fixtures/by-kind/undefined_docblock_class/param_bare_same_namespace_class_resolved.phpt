===description===
A bare `@param` class name referring to a same-namespace sibling class must
resolve against the current namespace, same as a native type hint would —
`resolve_union_doc` previously left every `@param` class name unqualified
(a leftover workaround meant only to protect real global builtins like
`Closure`), so `Calculator` and `App\Calculator` were treated as two
different, incompatible classes.
===config===
suppress=UnusedParam
===file===
<?php
namespace App;

class Calculator {}

class Registry {
    private static ?Calculator $instance = null;

    /**
     * @param Calculator|null $calculator
     */
    public function set($calculator): void {
        self::$instance = $calculator;
    }
}
===expect===
