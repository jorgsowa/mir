===description===
This var with bad type
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
InvalidReturnType@13:9-13:25: Return type 'int' is not compatible with declared 'string'
