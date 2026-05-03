===description===
require once global class fqn from namespaced
===file:Foo.php===
<?php
class Foo {}
===file:Main.php===
<?php
namespace App\Service;
require_once __DIR__ . '/Foo.php';
function run(): void {
    new \Foo();
}
===expect===
===ignore===
TODO
