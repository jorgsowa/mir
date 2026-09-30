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
//  ^^^^^^^^ UndefinedClass: Class B does not exist
===expect===
