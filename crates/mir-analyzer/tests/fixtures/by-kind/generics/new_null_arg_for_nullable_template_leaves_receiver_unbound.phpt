===description===
`new Box(null)` against `@param T|null` says nothing about T, so the receiver
stays unparameterized and is accepted wherever a `Box<X>` is expected. A
template bound by a real argument is still pinned and still checked.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
class Box {
    /** @param T|null $v */
    public function __construct(public $v = null) {}
}

/** @template T */
class Pair {
    /**
     * @param T|null $a
     * @param T $b
     */
    public function __construct(public $a, public $b) {}
}

/** @param Box<string> $b */
function takesStringBox(Box $b): void {}

/** @param Pair<string> $p */
function takesStringPair(Pair $p): void {}

takesStringBox(new Box(null));

$empty = new Box(null);
/** @mir-check $empty is Box */
takesStringBox($empty);

$bound = new Box('x');
/** @mir-check $bound is Box<string> */
takesStringBox($bound);

$int = new Box(1);
takesStringBox($int);
//             ^^^^ InvalidArgument: Argument $b of takesStringBox() expects 'Box<string>', got 'Box<int>'

$pair = new Pair(null, 'x');
/** @mir-check $pair is Pair<string> */
takesStringPair($pair);
