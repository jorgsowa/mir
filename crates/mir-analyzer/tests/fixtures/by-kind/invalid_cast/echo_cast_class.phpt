===description===
Echo cast class
===file===
<?php
class A {}
echo (string)(new A);
===expect===
InvalidCast@3:14-3:21: Cannot cast 'A' to 'string'
