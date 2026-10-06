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
UnusedSuppress@5:22-5:44: Suppress annotation for 'UndefinedPropertyFetch' is never used
