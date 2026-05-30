===description===
Nullable property fetch
===file===
<?php
$a = null;

echo $a->foo;
===expect===
NullPropertyFetch@4:6-4:13: Cannot access property $foo on null
