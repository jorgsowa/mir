===description===
References ignore var annotation
===config===
suppress=UnusedVariable
===file===
<?php
$a = 1;
/** @var int */
$b = &$a;

===expect===
