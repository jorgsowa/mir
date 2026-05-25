===description===
Echo cast class
===file===
<?php
class A {}
echo (string)(new A);
===expect===
InvalidCast
===ignore===
TODO
