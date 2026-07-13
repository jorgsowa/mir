===description===
`$cls::method()` where `$cls` holds a class-string variable now resolves the method, so a nonexistent one is reported UndefinedMethod.
===file===
<?php
class Foo {}

$cls = Foo::class;
$cls::missing();
===expect===
UndefinedMethod@5:0-5:15: Method Foo::missing() does not exist
