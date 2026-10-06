===description===
Nested array values are compared through the class hierarchy at any depth
===file===
<?php
class Animal {}
class Cat extends Animal {}

/** @param array<string, list<Cat>> $x @return array<string, list<Animal>> */
function listInMap(array $x): array { return $x; }

/** @param array<string, array<string, Cat>> $x @return array<string, array<string, Animal>> */
function mapInMap(array $x): array { return $x; }

/** @param array<string, array<int, list<Cat>>> $x @return array<string, array<int, list<Animal>>> */
function threeLevels(array $x): array { return $x; }

/** @param array{cats: list<Cat>} $x @return array<string, list<Animal>> */
function shapeInMap(array $x): array { return $x; }

/** @param list<non-empty-list<Cat>> $x @return list<list<Animal>> */
function listInList(array $x): array { return $x; }

/** @param array<string, list<Animal>> $x @return array<string, list<Cat>> */
function narrowingIsStillRejected(array $x): array { return $x; }
//                                                   ^^^^^^^^^^ InvalidReturnType: Return type 'array<string, list<Animal>>' is not compatible with declared 'array<string, list<Cat>>'

/** @param array<string, list<Cat>> $x @return array<int, list<Animal>> */
function outerKeyMismatchIsStillRejected(array $x): array { return $x; }
//                                                          ^^^^^^^^^^ InvalidReturnType: Return type 'array<string, list<Cat>>' is not compatible with declared 'array<int, list<Animal>>'

/** @param array<string, list<string>> $x @return array<string, list<Animal>> */
function unrelatedLeafIsStillRejected(array $x): array { return $x; }
//                                                       ^^^^^^^^^^ InvalidReturnType: Return type 'array<string, list<string>>' is not compatible with declared 'array<string, list<Animal>>'
