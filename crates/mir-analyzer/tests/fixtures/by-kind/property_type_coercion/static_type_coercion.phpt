===description===
Static type coercion
===config===
suppress=MissingPropertyType
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
PropertyTypeCoercion@8:8-8:23: Property $foo expects 'B|null', cannot assign 'A' — coercion may fail at runtime
