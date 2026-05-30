===description===
Deprecated class with extends
===file===
<?php
/**
 * @deprecated
 */
class Foo { }

class Bar extends Foo {}
===expect===
DeprecatedClass@7:0-7:24: Class Foo is deprecated
