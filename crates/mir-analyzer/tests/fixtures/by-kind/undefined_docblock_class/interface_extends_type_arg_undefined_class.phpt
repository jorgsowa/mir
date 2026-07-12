===description===
UndefinedDocblockClass fires when a class name inside an interface's own
`@extends` generic type-argument list does not exist.
===file===
<?php
/** @template T */
interface Box {}

/** @extends Box<NonExistentTypeArg> */
interface IntBox extends Box {}
===expect===
UndefinedDocblockClass@5:0-5:39: Docblock type 'NonExistentTypeArg' does not exist
