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
}
===expect===
Main.php: UndefinedClass@4:9-4:12: Class Foo does not exist
