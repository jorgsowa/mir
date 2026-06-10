===description===
Undefined static property assignment
===ignore===
TODO
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
