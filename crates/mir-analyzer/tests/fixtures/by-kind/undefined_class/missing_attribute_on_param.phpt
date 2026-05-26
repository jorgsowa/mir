===description===
Missing attribute on param
===file===
<?php
use FooBarPure;

function foo(#[Pure] string $str) : void {}
===expect===
UndefinedAttributeClass
===ignore===
TODO
