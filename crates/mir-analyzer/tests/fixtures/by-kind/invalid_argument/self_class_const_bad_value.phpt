===description===
Self class const bad value
===config===
suppress=UnusedParam
===file===
<?php
class A {
    const FOO = "foo";
    const BAR = "bar";

    /**
     * @param (self::FOO | self::BAR) $s
     */
    public static function foo(string $s) : void {}
}

A::foo("for");
//     ^^^^^ InvalidArgument: Argument $s of foo() expects '"foo"|"bar"', got '"for"'
===expect===
