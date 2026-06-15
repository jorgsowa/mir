===description===
Undefined callable method full string
===file===
<?php
class A {
    public static function bar(string $a): string {
        return $a . "b";
    }
}

function foo(callable $c): void {}

foo("A::barr");
===expect===
UnusedParam@8:13-8:24: Parameter $c is never used
UndefinedMethod@10:4-10:13: Method A::barr() does not exist
