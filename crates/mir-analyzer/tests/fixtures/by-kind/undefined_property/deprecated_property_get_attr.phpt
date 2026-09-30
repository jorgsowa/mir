===description===
Deprecated property get attr
===config===
suppress=MissingPropertyType
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
===expect===
