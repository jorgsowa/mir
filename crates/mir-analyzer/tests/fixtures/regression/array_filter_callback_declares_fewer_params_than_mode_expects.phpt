===description===
Same root cause as the array_map case: ARRAY_FILTER_USE_BOTH passes (value, key)
to the callback, but a callback is always free to declare fewer params and ignore
the rest — a value-only callback under USE_BOTH is valid PHP, not an arity error.
===file===
<?php
/** @var array<string, float> $arg */
$arg = [];
array_filter($arg, "strlen", ARRAY_FILTER_USE_BOTH);

===expect===
