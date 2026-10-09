===description===
Method call on a literal string
===file===
<?php
("hello")->someMethod();
//<^^^^^^^^^^^^^^^^^^^^^^^ InvalidMethodCall: Cannot call method someMethod() on non-object type '"hello"'
