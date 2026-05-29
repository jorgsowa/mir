===description===
Prevent extending interface
===file===
<?php
interface Foo {}

class Bar extends Foo {}
===expect===
UndefinedClass@4:19-4:22: Class Foo does not exist
