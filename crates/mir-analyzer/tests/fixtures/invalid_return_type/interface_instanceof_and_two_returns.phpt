===description===
interfaceInstanceofAndTwoReturns
===file===
<?php
interface A {}
interface B {}

class C implements A, B {}

function foo(A $i): B {
    if ($i instanceof B) {
        return $i;
    }

    return $i;
}

foo(new C);
===expect===
InvalidReturnStatement
===ignore===
TODO
