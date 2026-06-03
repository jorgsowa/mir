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
DeprecatedClass@7:6-7:9: Class Foo is deprecated
