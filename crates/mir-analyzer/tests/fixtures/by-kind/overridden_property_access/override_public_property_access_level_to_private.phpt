===description===
Override public property access level to private
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
}

class B extends A {
    /** @var string|null */
    private $foo;
//  ^^^^^^^^^^^^^ OverriddenPropertyAccess: Property B::$foo overrides with less visibility
}
===expect===
