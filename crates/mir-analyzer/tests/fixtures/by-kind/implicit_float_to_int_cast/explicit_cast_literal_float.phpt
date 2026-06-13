===description===
Explicit cast from float to int - user explicitly requests the cast, so no issue

===config===
suppress=UnusedVariable
===file===
<?php
$x = (int)3.7;

===expect===
