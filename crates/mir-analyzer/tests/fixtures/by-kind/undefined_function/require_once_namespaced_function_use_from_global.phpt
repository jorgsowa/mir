===description===
require once namespaced function use from global
===file:Helpers.php===
<?php
namespace Vendor\Lib;
function helper(): string {
    return 'ok';
}
===file:Main.php===
<?php
require_once __DIR__ . '/Helpers.php';
use function Vendor\Lib\helper;
function run(): string {
    return helper();
}
===expect===
