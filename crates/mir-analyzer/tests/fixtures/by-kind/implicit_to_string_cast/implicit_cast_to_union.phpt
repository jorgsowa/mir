===description===
No ImplicitToStringCast when class defines __toString and param is a union containing string
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
