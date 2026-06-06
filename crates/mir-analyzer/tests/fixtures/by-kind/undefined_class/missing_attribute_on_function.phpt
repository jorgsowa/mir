===description===
Missing attribute on function
===file===
<?php
use FooBarPure;

#[Pure]
function foo() : void {}
===expect===
ParseError@2:5-2:15: Parse error: The use statement with non-compound name 'FooBarPure' has no effect
UndefinedAttributeClass@4:3-4:7: Attribute class Pure does not exist
