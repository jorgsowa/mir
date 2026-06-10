===description===
Invalid docblock param default
===ignore===
TODO
===file===
<?php
/**
 * @param  int $p
 * @return void
 */
function f($p = false) {}
===expect===
