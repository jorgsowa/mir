===description===
Nullable property fetch
===file===
<?php
$a = null;

echo $a->foo;
===expect===
NullPropertyFetch@4:5-4:12: Cannot access property $foo on null
