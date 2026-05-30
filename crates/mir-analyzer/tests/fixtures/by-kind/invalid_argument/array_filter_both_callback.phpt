===description===
Array filter both callback
===file===
<?php
/** @var array<string, float> $arg */
$arg = [];
array_filter($arg, "strlen", ARRAY_FILTER_USE_BOTH);
===expect===
InvalidArgument@4:20-4:28: Argument $callback of array_filter() expects 'callable accepting 2 arguments', got 'callable accepting 1 argument'
