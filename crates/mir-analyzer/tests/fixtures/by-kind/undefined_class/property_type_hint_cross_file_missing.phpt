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
//<^^^^^^^^^^^ MissingConstructor: Class Bar has uninitialized properties but no constructor
    public Missing $prop;
//         ^^^^^^^ UndefinedClass: Class Vendor\Lib\Missing does not exist
}
===expect===
