===description===
require once namespaced class unqualified from global
===file:Foo.php===
<?php
namespace Vendor\Lib;
class Foo {}
===file:Main.php===
<?php
require_once __DIR__ . '/Foo.php';
function run(): void {
    new Foo();
//      ^^^ UndefinedClass: Class Foo does not exist
}
