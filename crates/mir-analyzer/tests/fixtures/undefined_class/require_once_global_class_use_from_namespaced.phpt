===description===
require once global class use from namespaced
===file:Foo.php===
<?php
class Foo {}
===file:Main.php===
<?php
namespace App\Service;
require_once __DIR__ . '/Foo.php';
use Foo;
function run(): void {
    new Foo();
}
===expect===
