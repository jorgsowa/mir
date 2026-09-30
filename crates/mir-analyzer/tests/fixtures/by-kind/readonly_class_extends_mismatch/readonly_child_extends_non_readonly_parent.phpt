===description===
A readonly class extending a non-readonly class is a PHP fatal.
===file===
<?php
class A {
    public function __construct(public int $x) {}
}

readonly class B extends A {
//       ^^^^^^^^^^^^^^^^^^^ ReadonlyClassExtendsMismatch: Readonly class B cannot extend non-readonly class A
    public function __construct(public int $y) {}
}
===expect===
