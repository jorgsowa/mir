===description===
References from a class declaration name include its use sites.
===cursor===
references include_declaration
===file===
<?php
class Fo<CURSOR>o {}
function f(Foo $m): Foo { return $m; }
===expect===
test.php@2:6-2:9
test.php@3:11-3:14
test.php@3:20-3:23
