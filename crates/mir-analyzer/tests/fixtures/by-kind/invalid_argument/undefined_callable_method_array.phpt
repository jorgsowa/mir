===description===
Undefined callable method array
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

foo([A::class, "::barr"]);
===expect===
UndefinedMethod@10:4-10:24: Method A::::barr() does not exist
