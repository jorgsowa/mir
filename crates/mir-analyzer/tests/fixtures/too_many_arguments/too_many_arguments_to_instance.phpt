===description===
tooManyArgumentsToInstance
===file===
<?php
class A {
    public function fooFoo(int $a): void {}
}

(new A)->fooFoo(5, "dfd");
===expect===
TooManyArguments
===ignore===
TODO
