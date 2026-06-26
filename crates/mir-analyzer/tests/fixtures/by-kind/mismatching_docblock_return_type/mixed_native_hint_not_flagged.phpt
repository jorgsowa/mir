===description===
MismatchingDocblockReturnType does NOT fire when the native hint is `mixed` —
the rule is suppressed when the hint resolves to mixed.
===file===
<?php
/** @return string */
function mixedHintStringDoc(): mixed { return 'x'; }

/** @return int */
function mixedHintIntDoc(): mixed { return 42; }
===expect===
