===description===
A readonly class extending a non-readonly class is a PHP fatal.
===file===
<?php
class A {
    public function __construct(public int $x) {}
}

readonly class B extends A {
    public function __construct(public int $y) {}
}
===expect===
ReadonlyClassExtendsMismatch@6:9-6:28: Readonly class B cannot extend non-readonly class A
