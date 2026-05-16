===description===
does not report namespaced function when called
===file===
<?php
namespace App;

function helper(): void {}

helper();
===expect===
===ignore===
TODO
