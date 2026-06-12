===description===
Undefined static property assignment
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
UnusedPsalmSuppress@6:0-6:0: Suppress annotation for 'UndefinedPropertyFetch' is never used
