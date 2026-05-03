===file:Helpers.php===
<?php
function helper(): string {
    return 'ok';
}
===file:Main.php===
<?php
namespace App\Service;
require_once __DIR__ . '/Helpers.php';
function run(): string {
    return helper();
}
===expect===
