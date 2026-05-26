===description===
Deprecated property set from inside the class
===file===
<?php
class A{
    /**
     * @deprecated
     * @var ?int
     */
    public $foo;
    public function bar(int $p): void
    {
        $this->foo = $p;
    }
}

===expect===
DeprecatedProperty
===ignore===
TODO
