===description===
SKIPPED-switchManyGetClassWithRepetitionWithProperLineNumber
===ignore===
TODO
===file===
<?php
class A {}
class B extends A {}
class C extends A {}
class D extends A {}

function foo(A $a) : void {
    switch(get_class($a)) {
        case B::class:
        case C::class:
        case B::class:
        case C::class:
        case D::class:
            echo "goodbye";
    }
}
===expect===
