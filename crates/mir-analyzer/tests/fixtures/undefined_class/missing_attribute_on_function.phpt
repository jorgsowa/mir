===description===
Missing attribute on function
===file===
<?php
use FooBarPure;

#[Pure]
function foo() : void {}
===expect===
UndefinedAttributeClass
===ignore===
TODO
