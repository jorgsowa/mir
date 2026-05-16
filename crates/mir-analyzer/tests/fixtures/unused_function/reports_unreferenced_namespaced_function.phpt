===description===
reports unreferenced namespaced function
===file===
<?php
namespace App;

function helper(): void {}
===expect===
UnusedFunction@4:0: Function helper() is never called
