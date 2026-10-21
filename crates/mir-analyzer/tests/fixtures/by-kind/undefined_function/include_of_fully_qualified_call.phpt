===description===
A fully qualified call inside an include expression resolves to the global function.
===file===
<?php
namespace App;

function load(): void {
    include \dirname(__DIR__) . '/view.php';
}
