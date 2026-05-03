===description===
require once namespaced class use from global
===file:Foo.php===
<?php
namespace Vendor\Lib;
class Foo {}
===file:Main.php===
<?php
require_once __DIR__ . '/Foo.php';
use Vendor\Lib\Foo;
function run(): void {
    new Foo();
}
===expect===
===ignore===
TODO
