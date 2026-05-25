===description===
Missing attribute on class
===file===
<?php
use FooBarPure;

#[Pure]
class Video {}
===expect===
UndefinedAttributeClass
===ignore===
TODO
