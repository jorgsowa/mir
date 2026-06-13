===description===
Too many arguments to instance
===config===
suppress=UnusedParam
===file===
<?php
class A {
    public function fooFoo(int $a): void {}
}

(new A)->fooFoo(5, "dfd");
===expect===
TooManyArguments@6:20-6:25: Too many arguments for fooFoo(): expected 1, got 2
