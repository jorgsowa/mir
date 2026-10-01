===description===
class-string<Foo> over a real class still reports undefined members
===file===
<?php
class Foo {
    public static function make(): static { return new static(); }
}

/** @param class-string<Foo> $c */
function f(string $c): void {
    /** @mir-check $c is class-string<Foo> */
    $c::make();
    $c::missing();
}
===expect===
UndefinedMethod@10:4-10:17: Method Foo::missing() does not exist
