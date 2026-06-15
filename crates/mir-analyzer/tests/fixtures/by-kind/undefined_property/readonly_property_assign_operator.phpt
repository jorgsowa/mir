===description===
Readonly property assign operator
===file===
<?php
class Test {
    /** @readonly */
    public int $prop;

    public function __construct(int $prop) {
        // Legal initialization.
        $this->prop = $prop;
    }
}

$test = new Test(5);

$test->prop += 1;
===expect===
ReadonlyPropertyAssignment@14:0-14:16: Cannot assign to readonly property Test::$prop outside of constructor
