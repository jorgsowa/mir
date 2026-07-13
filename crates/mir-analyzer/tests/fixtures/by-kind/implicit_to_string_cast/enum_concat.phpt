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
===expect===
ImplicitToStringCast@5:16-5:28: Class Suit is implicitly cast to string
