===description===
Deprecated property get
===config===
suppress=MissingPropertyType
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
===expect===
