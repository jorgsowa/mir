===description===
Deprecated property get
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
echo (new A)->foo;
//            ^^^ DeprecatedProperty: Property A::$foo is deprecated
