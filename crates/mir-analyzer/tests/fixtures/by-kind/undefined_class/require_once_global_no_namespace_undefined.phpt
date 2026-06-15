===description===
require once global no namespace undefined
===file:Helpers.php===
<?php
class Helper {}
===file:Main.php===
<?php
require_once __DIR__ . '/Helpers.php';
function run(): void {
    new Missing();
}
===expect===
Main.php: UndefinedClass@4:8-4:15: Class Missing does not exist
