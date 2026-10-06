===description===
UndefinedDocblockClass fires when a class name inside an `@extends`
generic type-argument list does not exist.
===file===
<?php
/** @template T */
class Box {}

/** @extends Box<NonExistentTypeArg> */
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedDocblockClass: Docblock type 'NonExistentTypeArg' does not exist
class IntBox extends Box {}
