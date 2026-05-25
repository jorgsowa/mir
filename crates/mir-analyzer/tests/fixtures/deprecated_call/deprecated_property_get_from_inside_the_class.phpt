===description===
Deprecated property get from inside the class
===file===
<?php
class A{
    /**
     * @deprecated
     * @var ?int
     */
    public $foo;
    public function bar(): void
    {
        echo $this->foo;
    }
}

===expect===
DeprecatedProperty
===ignore===
TODO
