===description===
undefinedCallableMethodFullString
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
UnusedParam@8:13: Parameter $c is never used
UndefinedMethod@10:4: Method A::barr() does not exist
