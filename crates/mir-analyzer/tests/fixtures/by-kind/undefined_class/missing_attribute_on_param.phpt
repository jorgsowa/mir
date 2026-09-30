===description===
Missing attribute on param
===config===
suppress=UnusedParam
===file===
<?php
function foo(#[Pure] string $str) : void {}
//             ^^^^ UndefinedAttributeClass: Attribute class Pure does not exist
===expect===
