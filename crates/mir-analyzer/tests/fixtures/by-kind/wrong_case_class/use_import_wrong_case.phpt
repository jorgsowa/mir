===description===
A use import with wrong class name casing is reported.
===config===
suppress=UnusedVariable
===file:Lib.php===
<?php

namespace Lib;

class MyClass {}
===file:Main.php===
<?php

use Lib\myClass;

$x = new myClass();
===expect===
Main.php: WrongCaseClass@3:4-3:15: Class name 'myClass' has incorrect casing; use 'MyClass'
Main.php: WrongCaseClass@5:9-5:16: Class name 'myClass' has incorrect casing; use 'MyClass'
