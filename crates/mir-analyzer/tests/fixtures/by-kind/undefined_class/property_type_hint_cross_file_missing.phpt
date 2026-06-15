===description===
property type hint cross file missing
===file:Dep.php===
<?php
namespace Vendor\Lib;
class Dep {}
===file:Main.php===
<?php
use Vendor\Lib\Missing;
class Bar {
    public Missing $prop;
}
===expect===
Main.php: MissingConstructor@3:0-3:11: Class Bar has uninitialized properties but no constructor
Main.php: UndefinedClass@4:11-4:18: Class Vendor\Lib\Missing does not exist
