===description===
Duplicate param
===config===
suppress=MissingParamType,UnusedParam
===file===
<?php
/**
 * @return void
 */
function f($p, $p) {}
===expect===
