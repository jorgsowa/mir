===description===
Missing attribute on function
===file===
<?php
#[Pure]
function foo() : void {}
===expect===
UndefinedAttributeClass@2:3-2:7: Attribute class Pure does not exist
