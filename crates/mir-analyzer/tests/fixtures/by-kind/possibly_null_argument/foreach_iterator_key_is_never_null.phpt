===description===
`foreach` never yields a null key, so a `key()` declared `string|int|null`
(ArrayIterator, user iterators, inherited `key()`, IteratorAggregate
delegating to one) gives a non-null `$k`. A direct `key()` call stays nullable.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedForeachValue errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function arrayIterator(ArrayIterator $it): void {
    foreach ($it as $k => $v) { strlen((string) $v); echo strlen($k); }
}

function newArrayIterator(): void {
    foreach (new ArrayIterator(['a' => 1]) as $k => $v) { echo strlen($k); }
}

class Bag implements Iterator {
    public function current(): int { return 1; }
    public function key(): ?string { return null; }
    public function next(): void {}
    public function rewind(): void {}
    public function valid(): bool { return false; }
}

class SubBag extends Bag {}

class Agg implements IteratorAggregate {
    public function getIterator(): Bag { return new Bag(); }
}

function userIterator(Bag $b, SubBag $s, Agg $a): void {
    foreach ($b as $k => $v) { echo strlen($k); }
    foreach ($s as $k => $v) { echo strlen($k); }
    foreach ($a as $k => $v) { echo strlen($k); }
}

function directKeyCall(Bag $b): void {
    echo strlen($b->key());
//              ^^^^^^^^^ PossiblyNullArgument: Argument $string of strlen() might be null
}
