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
//           ^^ ImplicitToStringCast: Class Suit is implicitly cast to string
===expect===
