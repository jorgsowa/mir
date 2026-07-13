===description===
FN: enum-case atomics were invisible to the string-interpolation
implicit-to-string check.
===config===
suppress=UnusedVariable
===file===
<?php
enum Suit {
    case Hearts;
}
$e = Suit::Hearts;
$s = "Suit: {$e}";
===expect===
ImplicitToStringCast@6:13-6:15: Class Suit is implicitly cast to string
