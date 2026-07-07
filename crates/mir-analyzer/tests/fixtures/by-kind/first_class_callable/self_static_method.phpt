===description===
P3: self::method(...) inside a class resolves to the class's own static method and
produces a typed Closure. The closure's return type resolves `self` to the
concrete receiver class (`Factory`), the same way a direct `self::create()`
call already does via substitute_static_in_return — not the raw, unresolved
`self(Factory)` wrapper the FCC path used to leave in place.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

class Factory {
    public static function create(string $name): self {
        return new self();
    }

    public static function getFactory(): \Closure {
        $fn = self::create(...);
        /** @mir-check $fn is Closure(string): Factory */
        $_ = $fn;
        return $fn;
    }
}
===expect===
