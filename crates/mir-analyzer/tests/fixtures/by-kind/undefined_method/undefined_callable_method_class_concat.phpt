===description===
Undefined callable method class concat
===ignore===
TODO
===file===
<?php
class A {
    public static function bar(string $a): string {
        return $a . "b";
    }
}

function foo(callable $c): void {}

foo(A::class . "::barr");
===expect===
