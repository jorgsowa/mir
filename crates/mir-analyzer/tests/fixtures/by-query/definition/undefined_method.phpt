===description===
An undefined method has no definition.
===cursor===
definition
===file===
<?php
final class User {}
(new User())->miss<CURSOR>ing();
===expect===
error: NotFound
