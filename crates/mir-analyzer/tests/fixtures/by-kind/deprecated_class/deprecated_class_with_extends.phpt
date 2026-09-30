===description===
Deprecated class with extends
===file===
<?php
/**
 * @deprecated
 */
class Foo { }

class Bar extends Foo {}
//<^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedClass: Class Foo is deprecated
===expect===
