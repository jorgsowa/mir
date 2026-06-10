===description===
Possibly bad static assignment
===ignore===
TODO
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
