===description===
Too few arguments to instance
===config===
suppress=UnusedParam
===file===
<?php
class A {
    public function fooFoo(int $a): void {}
}

(new A)->fooFoo();
===expect===
TooFewArguments@6:0-6:17: Too few arguments for fooFoo(): expected 1, got 0
