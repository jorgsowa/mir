===description===
Prevent abstract instantiation definite class
===file===
<?php
abstract class A {}

function foo(string $a_class) : void {
    if ($a_class === A::class) {
        // After narrowing to class-string<A>, new $a_class() is valid because
        // class-string<T> means the held name is a concrete subclass of T.
        new $a_class();
    }
}
===expect===
