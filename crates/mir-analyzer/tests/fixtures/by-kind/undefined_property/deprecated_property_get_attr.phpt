===description===
Deprecated property get attr
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
     * @var ?int
     */
    #[Deprecated]
    public $foo;
}
echo (new A)->foo;
//            ^^^ DeprecatedProperty: Property A::$foo is deprecated
