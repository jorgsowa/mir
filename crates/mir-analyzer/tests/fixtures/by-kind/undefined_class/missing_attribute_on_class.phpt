===description===
Missing attribute on class
===file===
<?php
use FooBarPure;

#[Pure]
class Video {}
===expect===
ParseError@2:5-2:15: Parse error: The use statement with non-compound name 'FooBarPure' has no effect
UndefinedAttributeClass@4:3-4:7: Attribute class Pure does not exist
