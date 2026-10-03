===description===
FN: enum-case atomics were invisible to the string-interpolation
implicit-to-string check.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
enum Suit {
    case Hearts;
}
$e = Suit::Hearts;
$s = "Suit: {$e}";
//           ^^ ImplicitToStringCast: Class Suit is implicitly cast to string
===expect===
