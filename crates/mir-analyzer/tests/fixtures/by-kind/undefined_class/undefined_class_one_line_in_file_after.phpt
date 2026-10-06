===description===
Undefined class one line in file after
===file===
<?php
/**
 * @suppress UndefinedClass
 */
new B();
new C();
//  ^ UndefinedClass: Class C does not exist
