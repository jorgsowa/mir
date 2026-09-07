===description===
`$this->obj::class === Foo::class` must narrow `$this->obj` to `Foo`, the
same as `get_class($this->obj) === Foo::class` already does — the
`$obj::class` extractor only ever matched a plain variable on the class
side, never a property access, so a property receiver fell through
unnarrowed. Covers both operand orders, the string-literal comparison
form, and the loose `==` form.
===config===
suppress=MissingConstructor,UnusedParam
===file===
<?php
class Foo {}

class Box {
    public object $obj;
}

function leftPropClassConst(Box $box): void {
    if ($box->obj::class === Foo::class) {
        /** @mir-check $box->obj is Foo */
        echo "";
    }
}

function rightPropClassConst(Box $box): void {
    if (Foo::class === $box->obj::class) {
        /** @mir-check $box->obj is Foo */
        echo "";
    }
}

function leftPropClassConstStringLiteral(Box $box): void {
    if ($box->obj::class === Foo::class) {
        /** @mir-check $box->obj is Foo */
        echo "";
    }
    if ($box->obj::class === 'Foo') {
        /** @mir-check $box->obj is Foo */
        echo "";
    }
}

function loosePropClassConst(Box $box): void {
    if ($box->obj::class == Foo::class) {
        /** @mir-check $box->obj is Foo */
        echo "";
    }
}
===expect===
