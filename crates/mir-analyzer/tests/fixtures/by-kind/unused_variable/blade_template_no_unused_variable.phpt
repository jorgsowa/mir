===description===
UnusedVariable is suppressed in .blade.php view template files
===file:resources/views/welcome.blade.php===
<?php
$name = "World";
echo "Hello";
===expect===
