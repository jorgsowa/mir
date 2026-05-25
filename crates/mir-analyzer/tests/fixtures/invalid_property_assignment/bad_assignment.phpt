===description===
Bad assignment
===file===
<?php
class A {
    /** @var string */
    public $foo;

    public function barBar(): void
    {
        $this->foo = 5;
    }
}
===expect===
InvalidPropertyAssignmentValue
===ignore===
TODO
