===description===
Invalid docblock param default
===file===
<?php
/**
 * @param  int $p
 * @return void
 */
function f($p = false) {}
===expect===
