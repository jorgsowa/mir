===description===
A malformed literal-string union with an unbalanced quote count
("foo"with" has 3 quotes) is quote-aware-parsed rather than crashing or
producing a bogus error; the type falls back to unchecked rather than
flagging a nonsensical pseudo-type name.
===config===
suppress=UnusedParam
===file===
<?php
namespace Ns;

/** @param "foo"with"|"bar"|1|2|3 $s */
function foo($s) : void {}
foo(4);
===expect===
