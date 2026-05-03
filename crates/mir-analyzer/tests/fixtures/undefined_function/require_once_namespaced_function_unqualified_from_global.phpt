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
Main.php: UndefinedFunction: Function helper() is not defined
