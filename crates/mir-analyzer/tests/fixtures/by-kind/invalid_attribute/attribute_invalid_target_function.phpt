===description===
Attribute invalid target function
===file===
<?php
#[Attribute]
function foo(): void {}

===expect===
InvalidAttribute
===ignore===
TODO
