===description===
Class method parameter violation
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
  /** @var int */
  private $foo;

    public function __construct(int &$foo) {
        $this->foo = &$foo;
        $foo = "hello";
    }
}

$bar = 5;
$a = new A($bar); // $bar is constrained to an int
$bar = null; // ReferenceConstraintViolation issue emitted
===expect===
