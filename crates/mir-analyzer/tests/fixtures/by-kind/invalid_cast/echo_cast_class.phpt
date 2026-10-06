===description===
Echo cast class
===file===
<?php
class A {}
echo (string)(new A);
//           ^^^^^^^ InvalidCast: Cannot cast 'A' to 'string'
