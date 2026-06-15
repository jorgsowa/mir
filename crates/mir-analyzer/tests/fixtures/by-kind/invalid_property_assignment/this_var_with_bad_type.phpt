===description===
This var with bad type
===config===
suppress=MissingPropertyType
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
InvalidReturnType@13:8-13:24: Return type 'int' is not compatible with declared 'string'
