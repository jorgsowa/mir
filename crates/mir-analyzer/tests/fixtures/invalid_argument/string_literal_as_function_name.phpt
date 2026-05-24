===description===
stringLiteralAsFunctionName
===file===
<?php
// The issue: a TLiteralString("trim") should NOT be resolved via Fqcn::from_str
// which tries to look it up as a class name. It should be resolved as a function.

$callback = "trim";  // TLiteralString("trim")
$result = array_map($callback, ["  hello  ", "  world  "]);
===expect===
