===description===
FP-K: PHP stubs define multiple constructor overloads for DatePeriod. The
ISO-8601 string form (1 required arg) must not emit TooFewArguments —
the arity minimum must be the minimum across all declared overloads.
===config===
php_version=8.2
===file===
<?php

// ISO 8601 repeating interval form — only 1 required arg across all DatePeriod overloads
$_ = new DatePeriod('R5/2023-01-01T00:00:00Z/P1D');
===expect===
