===description===
Undefined static property assignment
===file===
<?php
class A {
    public static function barBar(): void
    {
        /** @suppress UndefinedPropertyFetch */
//                    ^^^^^^^^^^^^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'UndefinedPropertyFetch' is never used
        self::$foo = 5;
    }
}
