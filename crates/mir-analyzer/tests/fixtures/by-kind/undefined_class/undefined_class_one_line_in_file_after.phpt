===description===
Undefined class one line in file after
===file===
<?php
/**
 * @suppress UndefinedClass
 */
new B();
new C();
===expect===
UndefinedClass@6:5-6:6: Class C does not exist
