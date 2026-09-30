===description===
Prevent extending interface
===file===
<?php
interface Foo {}

class Bar extends Foo {}
//                ^^^ UndefinedClass: Class Foo does not exist
===expect===
