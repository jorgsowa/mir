===description===
require once global no namespace undefined
===file:Helpers.php===
<?php
function helper(): string {
    return 'ok';
}
===file:Main.php===
<?php
require_once __DIR__ . '/Helpers.php';
function run(): void {
    missing_helper();
//  ^^^^^^^^^^^^^^^^ UndefinedFunction: Function missing_helper() is not defined
}
