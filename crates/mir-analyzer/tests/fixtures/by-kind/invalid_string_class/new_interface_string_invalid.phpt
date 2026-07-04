===description===
`new $x()` where $x is typed interface-string must error: an interface name can
never be instantiated, unlike class-string<AbstractClass> which may still hold a
concrete non-abstract subclass name at runtime.
===config===
suppress=MissingReturnType
===file===
<?php
interface Shape {}

function test(string $className) {
    /** @var interface-string<Shape> $className */
    new $className();
}
===expect===
InvalidStringClass@6:8-6:18: Dynamic class instantiation requires string or class-string type, got 'interface-string<Shape>'
