===description===
undefinedStaticPropertyAssignment
===file===
<?php
class A {
    public static function barBar(): void
    {
        /** @suppress UndefinedPropertyFetch */
        self::$foo = 5;
    }
}
===expect===
UndefinedPropertyAssignment
===ignore===
TODO
