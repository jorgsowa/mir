===description===
FN: enum-case atomics were invisible to the echo implicit-to-string check.
===file===
<?php
enum Suit {
    case Hearts;
}
echo Suit::Hearts;
===expect===
ImplicitToStringCast@5:5-5:17: Class Suit is implicitly cast to string
