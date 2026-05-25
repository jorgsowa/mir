===description===
Duplicate param
===file===
<?php
/**
 * @return void
 */
function f($p, $p) {}
===expect===
DuplicateParam
===ignore===
TODO
