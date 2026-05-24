===description===
arrayFilterBothCallback
===file===
<?php
/** @var array<string, float> $arg */
$arg = [];
array_filter($arg, "strlen", ARRAY_FILTER_USE_BOTH);
===expect===
InvalidArgument@4:20: Argument $callback of array_filter() expects 'callable accepting 2 arg(s)', got 'callable accepting 1 argument(s)'
