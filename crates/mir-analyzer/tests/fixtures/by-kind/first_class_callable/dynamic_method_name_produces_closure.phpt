===description===
`$obj->$name(...)` with a dynamic (non-identifier) method name still
produces a `Closure` — the method can't be statically resolved, but the
first-class-callable expression's result type must still be `Closure`,
not a generic `callable`.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

class Greeter {
    public function hello(): string {
        return 'hi';
    }
}

function wrap(Greeter $obj, string $name): void {
    $x = $obj->$name(...);
    /** @mir-check $x is Closure */
    $_ = $x;
}
===expect===
