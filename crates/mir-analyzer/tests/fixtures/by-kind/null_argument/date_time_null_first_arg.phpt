===description===
Date time null first arg
===config===
suppress=UnusedVariable
===file===
<?php
$date = new DateTime(null);
//                   ^^^^ NullArgument: Argument $datetime of DateTime::__construct() cannot be null
===expect===
