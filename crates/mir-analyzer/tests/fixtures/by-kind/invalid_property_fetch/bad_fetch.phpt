===description===
Bad fetch
===file===
<?php
$a = "hello";
echo $a->foo;
===expect===
InvalidPropertyFetch@3:5-3:12: Cannot fetch property on non-object type '"hello"'
