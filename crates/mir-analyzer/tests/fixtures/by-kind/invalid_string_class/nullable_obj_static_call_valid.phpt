===description===
nullable object ($obj: ?Foo) on left side of :: should not emit InvalidStringClass
===file===
<?php
class Foo {
    public static function bar(): void {}
}

function test(?Foo $obj): void {
    $obj::bar();
//  ^^^^ PossiblyNullMethodCall: Cannot call method bar() on possibly null value
}
===expect===
