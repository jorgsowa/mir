===description===
`self::$prop::class === Foo::class` must narrow `self::$prop` to `Foo`, the
same as `get_class(self::$prop) === Foo::class` already does — the
`$obj::class` extractor only handled a plain variable/property on the class
side, never a static property. Covers both operand orders, the
string-literal comparison form, and the loose `==` form.
===config===
suppress=MissingConstructor,UnusedParam,MissingPropertyType
===file===
<?php
class Foo {}

class Container {
    public static ?Foo $prop = null;
}

function leftStaticPropClassConst(): void {
    if (Container::$prop::class === Foo::class) {
        /** @mir-check Container::$prop is Foo */
        echo "";
    }
}

function rightStaticPropClassConst(): void {
    if (Foo::class === Container::$prop::class) {
        /** @mir-check Container::$prop is Foo */
        echo "";
    }
}

function stringLiteralStaticPropClassConst(): void {
    if (Container::$prop::class === 'Foo') {
        /** @mir-check Container::$prop is Foo */
        echo "";
    }
    if ('Foo' === Container::$prop::class) {
        /** @mir-check Container::$prop is Foo */
        echo "";
    }
}

function looseStaticPropClassConst(): void {
    if (Container::$prop::class == Foo::class) {
        /** @mir-check Container::$prop is Foo */
        echo "";
    }
    if ('Foo' == Container::$prop::class) {
        /** @mir-check Container::$prop is Foo */
        echo "";
    }
}
===expect===
