===description===
thisVarWithBadType
===file===
<?php
class A {
    /** @var int */
    public $a = 0;

    /** @var string */
    public $b = "";

    public function fooFoo(): string
    {
        list($this->a, $this->b) = ["a", "b"];

        return $this->a;
    }
}
===expect===
InvalidPropertyAssignmentValue
===ignore===
TODO
