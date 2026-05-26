===description===
$obj::method() with a typed object parameter should not error
===file===
<?php
class Foo {
    public static function bar(): string {
        return "hello";
    }
}

function test(Foo $obj): void {
    $obj::bar();
}
===expect===
