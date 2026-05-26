===description===
Union of object types on left side of :: should not error
===file===
<?php
class Foo {
    public static function bar(): void {}
}
class Baz {
    public static function bar(): void {}
}

function test(Foo|Baz $obj): void {
    $obj::bar();
}
===expect===
