===description===
require once global no namespace no error
===file:Helpers.php===
<?php
function helper(): string {
    return 'ok';
}
===file:Main.php===
<?php
require_once __DIR__ . '/Helpers.php';
function run(): string {
    return helper();
}
===expect===
