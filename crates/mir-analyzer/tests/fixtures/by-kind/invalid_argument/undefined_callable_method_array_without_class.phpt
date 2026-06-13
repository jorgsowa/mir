===description===
Undefined callable method array without class
===config===
suppress=UnusedParam
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
