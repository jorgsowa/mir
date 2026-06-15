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
===expect===
DeprecatedProperty@9:14-9:17: Property A::$foo is deprecated
