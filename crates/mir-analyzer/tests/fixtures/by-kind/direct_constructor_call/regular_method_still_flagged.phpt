===description===
$this->__construct() from a regular method (not __wakeup/__clone) is still flagged.
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {
    public function __construct(private int $x) {}

    public function reset(int $x): void {
        $this->__construct($x);
    }
}
===expect===
DirectConstructorCall@6:8-6:30: Cannot call constructor of Foo directly
