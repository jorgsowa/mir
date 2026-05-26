===description===
Too many arguments to instance
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
