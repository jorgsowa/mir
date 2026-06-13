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
UnsupportedReferenceUsage@4:1-4:9: Reference assignment is not supported
