===description===
The name token is found past attributes and modifiers.
===cursor===
definition
===file===
<?php
#[Attribute]
final class Fo<CURSOR>o extends Bar {}
class Bar {}
===expect===
test.php@3:6-3:30
