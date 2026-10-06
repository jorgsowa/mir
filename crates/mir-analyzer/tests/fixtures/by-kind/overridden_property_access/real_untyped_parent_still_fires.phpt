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
//  ^^^^^^^^^^^^^ OverriddenPropertyAccess: Property B::$foo overrides with less visibility
    private $bar;
//  ^^^^^^^^^^^^^ OverriddenPropertyAccess: Property B::$bar overrides with less visibility
}
