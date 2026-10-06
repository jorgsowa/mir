===description===
reports unreferenced namespaced function
===file===
<?php
namespace App;

function helper(): void {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedFunction: Function helper() is never called
