===description===
Echo cast class
===file===
<?php
class A {}
echo (string)(new A);
===expect===
InvalidCast@3:13-3:20: Cannot cast 'A' to 'string'
