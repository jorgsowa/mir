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
//      ^^^^^^^^^^^^^^^ PropertyTypeCoercion: Property $foo expects 'B|null', cannot assign 'A' — coercion may fail at runtime
    }
}

class B extends A {}
===expect===
