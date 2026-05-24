===description===
tooManyArgumentsToStatic
===file===
<?php
class A {
    public static function fooFoo(int $a): void {}
}

A::fooFoo(5, "dfd");
===expect===
TooManyArguments
===ignore===
TODO
