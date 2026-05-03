===description===
require once namespaced function unqualified from global
===file:Helpers.php===
<?php
namespace Vendor\Lib;
function helper(): string {
    return 'ok';
}
===file:Main.php===
<?php
require_once __DIR__ . '/Helpers.php';
function run(): void {
    helper();
}
===expect===
Main.php: UndefinedFunction@4:4: Function helper() is not defined
===ignore===
TODO
