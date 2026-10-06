===description===
MismatchingDocblockReturnType fires when the @return docblock declares a
supertype that is NOT a subtype of the native hint class.
===file===
<?php
class Animal {}
class Dog extends Animal {}

/** @return Animal */
function getDog(): Dog { return new Dog(); }
//       ^^^^^^ MismatchingDocblockReturnType: Docblock return type 'Animal' does not match inferred 'Dog'
