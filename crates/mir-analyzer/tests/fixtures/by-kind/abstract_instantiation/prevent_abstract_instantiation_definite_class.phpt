===description===
Prevent abstract instantiation definite class
===file===
<?php
abstract class A {}

function foo(string $a_class) : void {
    if ($a_class === A::class) {
        new $a_class();
    }
}
===expect===
AbstractInstantiation@6:12-6:20: Cannot instantiate abstract class A
