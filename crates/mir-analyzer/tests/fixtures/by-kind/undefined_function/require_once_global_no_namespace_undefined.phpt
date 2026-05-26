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
}
===expect===
Main.php: UndefinedFunction@4:5: Function missing_helper() is not defined
