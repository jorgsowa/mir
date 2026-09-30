===description===
FN: enum-case atomics were invisible to every implicit-to-string check —
only TNamedObject was matched.
===config===
suppress=UnusedVariable
===file===
<?php
enum Suit {
    case Hearts;
}
$s = 'Suit: ' . Suit::Hearts;
//              ^^^^^^^^^^^^ ImplicitToStringCast: Class Suit is implicitly cast to string
===expect===
