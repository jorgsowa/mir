===description===
Contradictory reference constraints
===config===
suppress=MissingPropertyType,UnusedVariable
===file===
<?php
class A {
    /** @var int */
    private $foo;

    public function __construct(int &$foo) {
        $this->foo = &$foo;
    }
}

class B {
    /** @var string */
    private $bar;

    public function __construct(string &$bar) {
        $this->bar = &$bar;
    }
}

if (rand(0, 1)) {
    $v = 5;
    $c = (new A($v)); // $v is constrained to an int
} else {
    $v = "hello";
    $c =  (new B($v)); // $v is constrained to a string
}

$v = 8;
===expect===
UnsupportedReferenceUsage@7:8-7:26: Reference assignment is not supported
UnsupportedReferenceUsage@16:8-16:26: Reference assignment is not supported
