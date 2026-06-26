===description===
MismatchingDocblockReturnType does NOT fire when the @return docblock narrows
the native hint to a subclass (subtype is compatible with the hint).
===file===
<?php
class Animal {}
class Dog extends Animal {}

/** @return Dog */
function getAnimal(): Animal { return new Dog(); }
===expect===
