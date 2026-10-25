===description===
A real PHP property without a type hint (has_native_type=false, from_docblock=false)
still establishes a visibility contract. Reducing visibility on such a property
must still emit OverriddenPropertyAccess.
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    /** @var string|null */
    public $foo;
    /** @var int */
    protected $bar;
}

class B extends A {
    private $foo;
//  ^^^^^^^^^^^^^ OverriddenPropertyAccess: Access level to B::$foo must be public (as in class A)
    private $bar;
//  ^^^^^^^^^^^^^ OverriddenPropertyAccess: Access level to B::$bar must be protected (as in class A) or weaker
}
