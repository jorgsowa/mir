===description===
Wrong case class name in nullable type hint is reported.
===file===
<?php
class User {}
function find(): ?user { return null; }
===expect===
WrongCaseClass@3:19-3:23: Class name 'user' has incorrect casing; use 'User'
