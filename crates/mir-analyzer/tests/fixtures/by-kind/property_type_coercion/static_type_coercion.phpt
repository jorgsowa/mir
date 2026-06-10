===description===
Static type coercion
===ignore===
TODO
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
