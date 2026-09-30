===description===
Deprecated class string constant
===file===
<?php
/**
 * @deprecated
 */
class Foo {}

echo Foo::class;
//   ^^^ DeprecatedClass: Class Foo is deprecated
===expect===
