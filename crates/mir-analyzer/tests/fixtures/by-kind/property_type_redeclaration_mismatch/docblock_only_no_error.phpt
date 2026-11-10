===description===
Docblock-only type mismatch is not flagged — only native type hints create runtime invariant
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    /** @var int */
    public $x = 1;
}

class B extends A {
    /** @var string */
    public $x = 'hello';
//  ^^^^^^^^^^^^^^^^^^^^ NonInvariantDocblockPropertyType: Docblock type 'string' of B::$x is not invariant with 'int' in class A
}
