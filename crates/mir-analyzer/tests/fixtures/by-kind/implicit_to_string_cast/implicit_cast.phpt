===description===
Implicit cast
===config===
suppress=UnusedParam
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
ImplicitToStringCast@11:7-11:14: Class A is implicitly cast to string
