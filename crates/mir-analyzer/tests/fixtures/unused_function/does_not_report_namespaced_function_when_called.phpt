===description===
does not report namespaced function when called
===config===
find_dead_code=true
===file===
<?php
namespace App;

function helper(): void {}

helper();
===expect===
===ignore===
TODO
