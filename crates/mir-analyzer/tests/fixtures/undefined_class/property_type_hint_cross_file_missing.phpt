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
Main.php: UndefinedClass: Class Vendor\Lib\Missing does not exist
