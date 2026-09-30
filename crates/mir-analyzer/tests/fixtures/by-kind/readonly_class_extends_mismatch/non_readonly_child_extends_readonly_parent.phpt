===description===
A non-readonly class extending a readonly class is a PHP fatal.
===file===
<?php
readonly class A {
    public function __construct(public int $x) {}
}

class B extends A {
//<^^^^^^^^^^^^^^^^^^^ ReadonlyClassExtendsMismatch: Non-readonly class B cannot extend readonly class A
    public function __construct(int $x) {
        parent::__construct($x);
    }
}
===expect===
