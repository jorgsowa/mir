===description===
Attribute invalid target parameter
===file===
<?php
function foo(#[Attribute] string $_bar): void {}

===expect===
InvalidAttribute
===ignore===
TODO
