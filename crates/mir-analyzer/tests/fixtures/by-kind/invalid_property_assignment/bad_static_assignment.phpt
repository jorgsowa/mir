===description===
Bad static assignment
===file===
<?php
class A {
    /** @var string */
    public static $foo = "a";

    public static function barBar(): void
    {
        self::$foo = 5;
    }
}
===expect===
InvalidPropertyAssignment@8:9-8:23: Property $foo expects 'string', cannot assign '5'
