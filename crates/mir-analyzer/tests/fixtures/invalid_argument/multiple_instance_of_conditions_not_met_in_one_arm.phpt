===description===
multipleInstanceOfConditionsNotMetInOneArm
===file===
<?php
interface Foo {}
class A implements Foo {}
class B implements Foo {}
class C {}

function baz(C $_): int {
    return 1;
}

function bar(Foo $foo): int {
    return match (true) {
        $foo instanceof A, $foo instanceof B => baz($foo),
        $foo instanceof C => 3,
        default => 0,
    };
}
===expect===
InvalidArgument
===ignore===
TODO
