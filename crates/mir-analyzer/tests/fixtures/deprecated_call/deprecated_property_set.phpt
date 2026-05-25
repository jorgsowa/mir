===description===
Deprecated property set
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
===expect===
DeprecatedProperty
===ignore===
TODO
