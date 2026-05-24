===description===
undefinedCallableMethodClass
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
UnusedParam@8:14: Parameter $c is never used
UndefinedClass@10:5: Class B does not exist
