===description===
preventExtendingInterface
===file===
<?php
interface Foo {}

class Bar extends Foo {}
===expect===
UndefinedClass@4:18: Class Foo does not exist
