===description===
$obj::method() arg types are still checked after object-FQCN resolution
===file===
<?php
class Foo {
    public static function bar(int $x): void { echo $x; }
}

function test(Foo $obj): void {
    $obj::bar("wrong");
}
===expect===
InvalidArgument@7:14-7:21: Argument $x of bar() expects 'int', got '"wrong"'
