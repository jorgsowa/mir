===description===
Nullable property fetch
===file===
<?php
$a = null;

echo $a->foo;
//   ^^^^^^^ NullPropertyFetch: Cannot access property $foo on null
