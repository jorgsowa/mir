===description===
Enum::Case assignment resolves to the enum type, not mixed
===file===
<?php
enum Suit { case Hearts; case Spades; }
$s = Suit::Hearts;
/** @mir-check $s is Suit */
echo $s->name;
===expect===
