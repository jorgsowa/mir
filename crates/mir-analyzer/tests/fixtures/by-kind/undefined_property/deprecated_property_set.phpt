===description===
Deprecated property set
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A{
    /**
     * @deprecated
     * @var ?int
     */
    public $foo;
}
$a = new A;
$a->foo = 5;
//<^^^^^^^^^^^ DeprecatedProperty: Property A::$foo is deprecated
===expect===
