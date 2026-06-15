===description===
Deprecated class string constant
===file===
<?php
/**
 * @deprecated
 */
class Foo {}

echo Foo::class;
===expect===
DeprecatedClass@7:5-7:8: Class Foo is deprecated
