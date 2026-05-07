===description===
arrayFilterBothCallback
===file===
<?php
/** @var array<string, float> $arg */
$arg = [];
array_filter($arg, "strlen", ARRAY_FILTER_USE_BOTH);
===expect===
