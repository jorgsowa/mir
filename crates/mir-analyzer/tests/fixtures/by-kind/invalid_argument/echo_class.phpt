===description===
Echo class
===config===
suppress=ImplicitToStringCast
===file===
<?php
class A {}
echo (new A);
===expect===
