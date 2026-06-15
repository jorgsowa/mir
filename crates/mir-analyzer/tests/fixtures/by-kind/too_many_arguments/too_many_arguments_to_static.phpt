===description===
Too many arguments to static
===config===
suppress=UnusedParam
===file===
<?php
class A {
    public static function fooFoo(int $a): void {}
}

A::fooFoo(5, "dfd");
===expect===
TooManyArguments@6:13-6:18: Too many arguments for fooFoo(): expected 1, got 2
