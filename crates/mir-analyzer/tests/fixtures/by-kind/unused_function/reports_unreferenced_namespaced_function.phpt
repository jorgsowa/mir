===description===
reports unreferenced namespaced function
===file===
<?php
namespace App;

function helper(): void {}
===expect===
UnusedFunction@4:0-4:26: Function helper() is never called
