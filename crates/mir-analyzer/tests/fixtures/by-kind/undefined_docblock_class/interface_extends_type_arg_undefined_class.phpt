===description===
UndefinedDocblockClass fires when a class name inside an interface's own
`@extends` generic type-argument list does not exist.
===file===
<?php
/** @template T */
interface Box {}

/** @extends Box<NonExistentTypeArg> */
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedDocblockClass: Docblock type 'NonExistentTypeArg' does not exist
interface IntBox extends Box {}
