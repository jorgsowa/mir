===description===
Class constant no class
===config===
suppress=UnusedParam
===file===
<?php
namespace Ns;

/** @param "foo"|"bar"|C::A|C::B $s */
function foo($s) : void {}
===expect===
