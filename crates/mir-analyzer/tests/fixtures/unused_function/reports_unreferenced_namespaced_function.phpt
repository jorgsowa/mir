===config===
find_dead_code=true
===file===
<?php
namespace App;

function helper(): void {}
===expect===
UnusedFunction: Function helper() is never called
