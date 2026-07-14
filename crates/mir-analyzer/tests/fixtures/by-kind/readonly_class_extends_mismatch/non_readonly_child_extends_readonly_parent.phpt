===description===
A non-readonly class extending a readonly class is a PHP fatal.
===file===
<?php
readonly class A {
    public function __construct(public int $x) {}
}

class B extends A {
    public function __construct(int $x) {
        parent::__construct($x);
    }
}
===expect===
ReadonlyClassExtendsMismatch@6:0-6:19: Non-readonly class B cannot extend readonly class A
