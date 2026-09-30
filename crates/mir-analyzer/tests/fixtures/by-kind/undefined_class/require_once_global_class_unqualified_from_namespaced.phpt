===description===
require once global class unqualified from namespaced
===file:Foo.php===
<?php
class Foo {}
===file:Main.php===
<?php
namespace App\Service;
require_once __DIR__ . '/Foo.php';
function run(): void {
    new Foo();
//      ^^^ UndefinedClass: Class App\Service\Foo does not exist
}
===expect===
