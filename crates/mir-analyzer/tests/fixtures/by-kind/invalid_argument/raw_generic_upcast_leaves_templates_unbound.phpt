===description===
A raw (unparameterized) instance of a generic class says nothing about its
type arguments, so upcasting it to a parameterized ancestor treats them as
unknown instead of filling in each template's bound (`array-key` for
ArrayIterator's TKey), which rejected any string-keyed expectation.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param Traversable<string, int> $t */
function takesStringKeyed(Traversable $t): void {}

/** @param Iterator<string, SplFileInfo> $t */
function takesFiles(Iterator $t): void {}

class Inner extends ArrayIterator {}

/**
 * @template K of array-key
 * @template-covariant V of object
 * @implements IteratorAggregate<K, V>
 */
abstract class Bag implements IteratorAggregate {}

/** @param IteratorAggregate<string, SplFileInfo> $t */
function takesBag(IteratorAggregate $t): void {}

function raw(ArrayIterator $it, Inner $inner, Bag $bag): void {
    takesStringKeyed($it);
    takesFiles($inner);
    takesFiles(new Inner());
    takesBag($bag);
}

/** @param Bag<int, SplFileInfo> $bag */
function boundMismatchStillCaught(Bag $bag): void {
    takesBag($bag);
}
===expect===
InvalidArgument@29:13-29:17: Argument $t of takesBag() expects 'IteratorAggregate<string, SplFileInfo>', got 'Bag<int, SplFileInfo>'
