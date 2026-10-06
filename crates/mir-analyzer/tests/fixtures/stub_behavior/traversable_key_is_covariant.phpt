===description===
Traversable/Iterator/IteratorAggregate key is covariant: a keyed Traversable satisfies a wider iterable
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param iterable<mixed> $items */
function takesIterable(iterable $items): void {}

/** @param iterable<int|string, string> $items */
function takesKeyed(iterable $items): void {}

/** @param iterable<int, int> $items */
function takesInts(iterable $items): void {}

/**
 * @param Traversable<int, string> $t
 * @param Iterator<int, string> $i
 * @param IteratorAggregate<int, string> $a
 * @param Generator<int, string> $g
 */
function accepts(Traversable $t, Iterator $i, IteratorAggregate $a, Generator $g): void {
    takesIterable($t);
    takesIterable($i);
    takesIterable($a);
    takesIterable($g);
    takesKeyed($t);
    takesKeyed($i);
}

/** @param Traversable<int, string> $t */
function rejectsWrongValue(Traversable $t): void {
    takesInts($t);
//            ^^ InvalidArgument: Argument $items of takesInts() expects 'iterable<int, int>', got 'Traversable<int, string>'
}
