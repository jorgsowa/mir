===description===
require once namespaced class fqn from global
===file:Foo.php===
<?php
namespace Vendor\Lib;
class Foo {}
===file:Main.php===
<?php
require_once __DIR__ . '/Foo.php';
function run(): void {
    new \Vendor\Lib\Foo();
}
===expect===
===ignore===
TODO
