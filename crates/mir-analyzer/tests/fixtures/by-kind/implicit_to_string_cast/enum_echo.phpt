===description===
FN: enum-case atomics were invisible to the echo implicit-to-string check.
===file===
<?php
enum Suit {
    case Hearts;
}
echo Suit::Hearts;
//   ^^^^^^^^^^^^ ImplicitToStringCast: Class Suit is implicitly cast to string
