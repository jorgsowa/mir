===description===
An enum case's ->value used as an array index keeps the literal key on write 
and on read, like a literal-string key. A non-enum dynamic key still widens.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
enum Suit: string { case Hearts = 'h'; case Spades = 's'; }
enum Rank: int { case One = 1; }

function run(string $dyn): void {
    $a = [];
    $a[Suit::Hearts->value] = 1;
    /** @mir-check $a is array{'h': 1} */
    $_ = $a;
    $a[Suit::Spades->value] = 2;
    /** @mir-check $a is array{'h': 1, 's': 2} */
    $_ = $a;
    /** @mir-check $a[Suit::Hearts->value] is 1 */
    $_ = $a[Suit::Hearts->value];

    $n = [];
    $n[Rank::One->value] = 'x';
    /** @mir-check $n is array{1: "x"} */
    $_ = $n;

    $d = [];
    $d[$dyn] = 1;
    /** @mir-check $d is array<string, 1> */
    $_ = $d;
}
===expect===
