===description===
A call to an undefined method still resolves to the call, typed mixed.
===cursor===
symbol
===file===
<?php
final class User {}
(new User())->miss<CURSOR>ing();
===expect===
kind: method call User::missing
type: mixed
