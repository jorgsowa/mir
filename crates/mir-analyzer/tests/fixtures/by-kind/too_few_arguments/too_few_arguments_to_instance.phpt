===description===
Too few arguments to instance
===file===
<?php
class A {
    public function fooFoo(int $a): void {}
}

(new A)->fooFoo();
===expect===
TooFewArguments@6:1-6:18: Too few arguments for fooFoo(): expected 1, got 0
