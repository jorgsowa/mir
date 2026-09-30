===description===
Bad fetch
===file===
<?php
$a = "hello";
echo $a->foo;
//   ^^^^^^^ InvalidPropertyFetch: Cannot fetch property on non-object type '"hello"'
===expect===
