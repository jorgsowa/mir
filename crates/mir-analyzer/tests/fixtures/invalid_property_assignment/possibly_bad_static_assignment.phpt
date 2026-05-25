===description===
Possibly bad static assignment
===file===
<?php
class A {
    /** @var string */
    public static $foo = "a";

    public function barBar(): void
    {
        self::$foo = rand(0, 1) ? 5 : "hello";
    }
}
===expect===
PossiblyInvalidPropertyAssignmentValue
===ignore===
TODO
