===description===
Implicit cast
===file===
<?php
class A {
    public function __toString(): string
    {
        return "hello";
    }
}

/** @mutation-free */
function fooFoo(string $b): void {}
fooFoo(new A());
===expect===
ImplicitToStringCast
===ignore===
TODO
