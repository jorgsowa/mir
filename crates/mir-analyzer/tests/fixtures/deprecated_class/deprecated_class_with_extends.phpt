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
DeprecatedClass
===ignore===
TODO
