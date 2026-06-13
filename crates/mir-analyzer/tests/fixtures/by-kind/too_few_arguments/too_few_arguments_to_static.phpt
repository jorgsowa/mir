===description===
Too few arguments to static
===config===
suppress=UnusedParam
===file===
<?php
class A {
    public static function fooFoo(int $a): void {}
}

A::fooFoo();
===expect===
TooFewArguments@6:1-6:12: Too few arguments for fooFoo(): expected 1, got 0
