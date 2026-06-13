===description===
Static mixed type coercion
===config===
suppress=MissingPropertyType
===file===
<?php
class A {
    /** @var array<int, A> */
    public static $foo = [];

    /** @param A[] $arr */
    public static function barBar(array $arr): void
    {
        self::$foo = $arr;
    }
}
===expect===
