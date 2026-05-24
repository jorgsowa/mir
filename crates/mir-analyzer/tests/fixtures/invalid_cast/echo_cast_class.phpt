===description===
echoCastClass
===file===
<?php
class A {}
echo (string)(new A);
===expect===
InvalidCast
===ignore===
TODO
