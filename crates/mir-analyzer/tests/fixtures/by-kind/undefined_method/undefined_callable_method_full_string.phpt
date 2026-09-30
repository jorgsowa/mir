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
//           ^^^^^^^^^^^ UnusedParam: Parameter $c is never used

foo("A::barr");
//  ^^^^^^^^^ UndefinedMethod: Method A::barr() does not exist
===expect===
