===description===
Undefined static property assignment. mir reports reads and writes as one `UndefinedProperty`
kind, so Psalm's `UndefinedPropertyFetch` suppresses the write too.
===file===
<?php
class A {
    public static function barBar(): void
    {
        /** @suppress UndefinedPropertyFetch */
        self::$foo = 5;
    }
}
