===description===
Class method parameter violation in post assignment
===ignore===
TODO
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
