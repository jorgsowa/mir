===description===
undefinedClassOneLineInFileAfter
===file===
<?php
/**
 * @psalm-suppress UndefinedClass
 */
new B();
new C();
===expect===
UndefinedClass@6:4: Class C does not exist
