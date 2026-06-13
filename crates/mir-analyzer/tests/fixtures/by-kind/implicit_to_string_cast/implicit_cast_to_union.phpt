===description===
Implicit cast to union
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

/**
 * @param string|int $b
 * @mutation-free
 */
function fooFoo($b): void {}
fooFoo(new A());
===expect===
ImplicitToStringCast@14:8-14:15: Class A is implicitly cast to string
