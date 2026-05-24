===description===
tooFewArgumentsToStatic
===file===
<?php
class A {
    public static function fooFoo(int $a): void {}
}

A::fooFoo();
===expect===
TooFewArguments
===ignore===
TODO
