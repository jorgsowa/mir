===description===
implicitCastToUnion
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
ImplicitToStringCast
===ignore===
TODO
