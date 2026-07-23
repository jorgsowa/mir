===description===
Two suppression-directive keywords in one comment used to resolve to
whichever KEYWORDS table entry happened to match first, not whichever
keyword actually appears leftmost in the text — so the author's own
@mir-ignore-line (naming UndefinedFunction specifically, targeting its
own line) was silently dropped in favor of a later, textually-second
@phpstan-ignore-next-line (a blanket suppression of the NEXT line),
because that entry sits earlier in the fixed table. The leftmost keyword
in the text must win: foo() (this line) should be suppressed, bar() (the
next line) should not.
===config===
===file===
<?php
foo(); // @mir-ignore-line UndefinedFunction @phpstan-ignore-next-line
bar();
===expect===
UndefinedFunction@3:0-3:5: Function bar() is not defined
