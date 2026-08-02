===description===
`Foo::$name(...)` with a dynamic (non-identifier) static method name still
produces a `Closure`, matching the instance-method dynamic-name case.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

class Greeter {
    public static function hello(): string {
        return 'hi';
    }
}

function wrap(string $name): void {
    $x = Greeter::$name(...);
    /** @mir-check $x is Closure */
    $_ = $x;
}
===expect===
