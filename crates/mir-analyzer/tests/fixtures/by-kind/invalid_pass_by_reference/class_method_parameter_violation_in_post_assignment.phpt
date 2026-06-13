===description===
Class method parameter violation in post assignment
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

$bar = 5;
$a = new A($bar);
$bar = null;
===expect===
UnsupportedReferenceUsage@7:9-7:27: Reference assignment is not supported
