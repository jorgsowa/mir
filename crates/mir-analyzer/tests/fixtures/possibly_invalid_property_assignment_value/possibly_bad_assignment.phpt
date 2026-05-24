===description===
possiblyBadAssignment
===file===
<?php
class A {
    /** @var string */
    public $foo;

    public function barBar(): void
    {
        $this->foo = rand(0, 1) ? 5 : "hello";
    }
}
===expect===
PossiblyInvalidPropertyAssignmentValue
===ignore===
TODO
