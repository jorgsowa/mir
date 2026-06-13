===description===
Invalid docblock param default
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param  int $p
 * @return void
 */
function f($p = false) {}
===expect===
