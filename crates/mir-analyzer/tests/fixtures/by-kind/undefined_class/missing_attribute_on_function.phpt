===description===
Missing attribute on function
===file===
<?php
#[Pure]
function foo() : void {}
===expect===
UndefinedAttributeClass@2:2-2:6: Attribute class Pure does not exist
