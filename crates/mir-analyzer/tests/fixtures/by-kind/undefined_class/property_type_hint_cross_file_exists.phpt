===description===
property type hint cross file exists
===file:Dep.php===
<?php
namespace Vendor\Lib;
class Dep {}
===file:Main.php===
<?php
use Vendor\Lib\Dep;
class Bar {
//<^^^^^^^^^^^ MissingConstructor: Class Bar has uninitialized properties but no constructor
    public Dep $prop;
}
===expect===
