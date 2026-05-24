===description===
duplicateParam
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
