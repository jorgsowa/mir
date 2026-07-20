===description===
`get_class(self::$prop) === Foo::class` (and `'Foo'` string-literal form,
either operand order, strict and loose `==`) narrows the static property
like its var/instance-property counterparts already do.
===config===
suppress=MissingConstructor,UnusedParam,PossiblyNullArgument
===file===
<?php
class Foo {
    public function bar(): void {}
}
class Container {
    private static ?Foo $prop = null;

    // Positive: ClassConstAccess form, both operand orders.
    public static function testClassConst(): void {
        if (get_class(self::$prop) === Foo::class) {
            self::$prop->bar();
        }
        if (Foo::class === get_class(self::$prop)) {
            self::$prop->bar();
        }
    }

    // Positive: string-literal form, strict and loose, both operand orders.
    public static function testStringLiteral(): void {
        if (get_class(self::$prop) === 'Foo') {
            self::$prop->bar();
        }
        if ('Foo' === get_class(self::$prop)) {
            self::$prop->bar();
        }
        if (get_class(self::$prop) == 'Foo') {
            self::$prop->bar();
        }
        if ('Foo' == get_class(self::$prop)) {
            self::$prop->bar();
        }
    }
}
===expect===
