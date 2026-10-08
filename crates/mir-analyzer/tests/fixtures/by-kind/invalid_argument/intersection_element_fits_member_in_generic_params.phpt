===description===
An `A&B` element is accepted where `A` is expected inside array, list, shape,
iterable and Traversable params; an intersection with no matching member is not,
and an invariant generic stays strict.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface A {}
interface B {}
interface C {}

/** @template T */
final class Box {}

/** @param iterable<A> $x */
function takesIterable(iterable $x): void {}
/** @param array<A> $x */
function takesArray(array $x): void {}
/** @param list<A> $x */
function takesList(array $x): void {}
/** @param \Traversable<int, A> $x */
function takesTraversable(\Traversable $x): void {}
/** @param array<C> $x */
function takesOtherArray(array $x): void {}
/** @param Box<A> $x */
function takesBox(Box $x): void {}

/** @param iterable<A&B> $y */
function iterableOfBoth(iterable $y): void {
    takesIterable($y);
}

/** @param array<A&B> $y */
function arrayOfBoth(array $y): void {
    takesArray($y);
    takesIterable($y);
}

/** @param list<A&B> $y */
function listOfBoth(array $y): void {
    takesList($y);
    takesArray($y);
}

/** @param \Traversable<int, A&B> $y */
function traversableOfBoth(\Traversable $y): void {
    takesTraversable($y);
    takesIterable($y);
}

function shapeOfBoth(A&B $y): void {
    takesIterable([$y]);
    takesList([$y, $y]);
}

/** @param array<A&B> $y */
function noMemberMatches(array $y): void {
    takesOtherArray($y);
//                  ^^ InvalidArgument: Argument $x of takesOtherArray() expects 'array<int|string, C>', got 'array<int|string, A&B>'
}

/** @param Box<A&B> $y */
function invariantGenericStaysStrict(Box $y): void {
    takesBox($y);
//           ^^ InvalidArgument: Argument $x of takesBox() expects 'Box<A>', got 'Box<A&B>'
}