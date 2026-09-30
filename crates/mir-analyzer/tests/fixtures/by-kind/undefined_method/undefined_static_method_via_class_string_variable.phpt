===description===
`$cls::method()` where `$cls` holds a class-string variable now resolves the method, so a nonexistent one is reported UndefinedMethod.
===file===
<?php
class Foo {}

$cls = Foo::class;
$cls::missing();
//<^^^^^^^^^^^^^^^ UndefinedMethod: Method Foo::missing() does not exist
===expect===
