===description===
undefinedClassOneLineInFileAfter
===file===
<?php
/**
 * @suppress UndefinedClass
 */
new B();
new C();
===expect===
UndefinedClass@6:5: Class C does not exist
