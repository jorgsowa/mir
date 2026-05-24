===description===
undefinedCallableMethodArrayWithoutClass
===file===
<?php
class A {
    public static function bar(string $a): string {
        return $a . "b";
    }
}

function foo(callable $c): void {}

foo(["A", "::barr"]);
===expect===
InvalidArgument
===ignore===
TODO
