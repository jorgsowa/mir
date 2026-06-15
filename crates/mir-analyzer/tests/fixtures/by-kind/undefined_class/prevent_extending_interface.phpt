===description===
Prevent extending interface
===file===
<?php
interface Foo {}

class Bar extends Foo {}
===expect===
UndefinedClass@4:18-4:21: Class Foo does not exist
