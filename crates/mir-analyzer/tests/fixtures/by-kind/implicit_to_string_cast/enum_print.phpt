===description===
FN: enum-case atomics were invisible to the print implicit-to-string check.
===file===
<?php
enum Suit {
    case Hearts;
}
print Suit::Hearts;
//    ^^^^^^^^^^^^ ImplicitToStringCast: Class Suit is implicitly cast to string
===expect===
