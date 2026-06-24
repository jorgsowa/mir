===description===
P3: self::method(...) inside a class resolves to the class's own static method and
produces a typed Closure.
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
        /** @mir-check $fn is Closure(string): self(Factory) */
        $_ = $fn;
        return $fn;
    }
}
===expect===
