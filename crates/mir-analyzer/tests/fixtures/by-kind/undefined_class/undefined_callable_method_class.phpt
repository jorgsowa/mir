===description===
Undefined callable method class
===config===
suppress=UnusedParam,UnusedFunction
===file===
<?php
class A {
    public static function bar(string $a): string {
        return $a . "b";
    }
}

function foo(callable $c): void {}

foo("B::bar");
===expect===
UndefinedClass@10:4-10:12: Class B does not exist
