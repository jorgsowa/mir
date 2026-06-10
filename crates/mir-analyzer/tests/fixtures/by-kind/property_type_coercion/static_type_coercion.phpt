===description===
Static type coercion
===file===
<?php
class A {
    /** @var B|null */
    public static $foo;

    public static function barBar(A $a): void
    {
        self::$foo = $a;
    }
}

class B extends A {}
===expect===
InvalidPropertyAssignment@8:9-8:24: Property $foo expects 'B|null', cannot assign 'A'
