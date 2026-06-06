===description===
Missing attribute on param
===file===
<?php
use FooBarPure;

function foo(#[Pure] string $str) : void {}
===expect===
ParseError@2:5-2:15: Parse error: The use statement with non-compound name 'FooBarPure' has no effect
UndefinedAttributeClass@4:16-4:20: Attribute class Pure does not exist
