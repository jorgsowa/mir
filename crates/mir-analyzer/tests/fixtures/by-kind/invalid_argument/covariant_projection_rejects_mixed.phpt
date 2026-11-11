===description===
A call-site covariant projection (`Collection<covariant Animal>`) rejects `Collection<mixed>`, as it rejects `Collection<int>`. A collection with no type arguments written, one whose template bound fits, and a star projection still pass.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Animal {}
final class Cat extends Animal {}

/** @template T */
final class Collection {}

/** @template T of Animal */
final class Zoo {}

/** @param Collection<covariant Animal> $animals */
function takesCovariant(Collection $animals): void {}

/** @param Zoo<covariant Animal> $zoo */
function takesZoo(Zoo $zoo): void {}

/** @param Collection<*> $any */
function takesAny(Collection $any): void {}

/**
 * @param Collection<Cat> $cats
 * @param Collection<mixed> $mixeds
 * @param Collection<int> $ints
 */
function example(Collection $cats, Collection $mixeds, Collection $ints, Collection $bare, Zoo $zoo): void
{
    takesCovariant($cats);
    takesCovariant($mixeds);
//                 ^^^^^^^ InvalidArgument: Argument $animals of takesCovariant() expects 'Collection<covariant Animal>', got 'Collection'
    takesCovariant($ints);
//                 ^^^^^ InvalidArgument: Argument $animals of takesCovariant() expects 'Collection<covariant Animal>', got 'Collection<int>'
    takesCovariant($bare);
    takesZoo($zoo);
    takesAny($mixeds);
    takesAny($bare);
}
