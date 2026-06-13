===description===
Missing attribute on param
===config===
suppress=UnusedParam
===file===
<?php
function foo(#[Pure] string $str) : void {}
===expect===
UndefinedAttributeClass@2:16-2:20: Attribute class Pure does not exist
