===description===
preventExtendingInterface
===file===
<?php
interface Foo {}

class Bar extends Foo {}
===expect===
UndefinedClass@4:19: Class Foo does not exist
