===description===
A namespaced function declaration name resolves to itself.
===cursor===
definition
===file===
<?php
namespace App;
function ba<CURSOR>r(): void {}
===expect===
test.php@3:0-3:23
