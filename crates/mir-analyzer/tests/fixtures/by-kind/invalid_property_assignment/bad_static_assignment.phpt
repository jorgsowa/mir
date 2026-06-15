===description===
Bad static assignment
===config===
suppress=MissingPropertyType
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
InvalidPropertyAssignment@8:8-8:22: Property $foo expects 'string', cannot assign '5'
