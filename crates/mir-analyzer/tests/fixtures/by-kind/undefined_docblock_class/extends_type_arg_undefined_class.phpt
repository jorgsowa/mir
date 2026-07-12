===description===
UndefinedDocblockClass fires when a class name inside an `@extends`
generic type-argument list does not exist.
===file===
<?php
/** @template T */
class Box {}

/** @extends Box<NonExistentTypeArg> */
class IntBox extends Box {}
===expect===
UndefinedDocblockClass@5:0-5:39: Docblock type 'NonExistentTypeArg' does not exist
