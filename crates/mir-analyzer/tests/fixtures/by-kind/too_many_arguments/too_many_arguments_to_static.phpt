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
//           ^^^^^ TooManyArguments: Too many arguments for fooFoo(): expected 1, got 2
===expect===
