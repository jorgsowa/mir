===file:Dep.php===
<?php
namespace Vendor\Lib;
class Dep {}
===file:Main.php===
<?php
use Vendor\Lib\Dep;
class Bar {
    public Dep $prop;
}
===expect===
