===description===
Bad fetch
===file===
<?php
$a = "hello";
echo $a->foo;
===expect===
InvalidPropertyFetch@3:6-3:13: Cannot fetch property on non-object type '"hello"'
