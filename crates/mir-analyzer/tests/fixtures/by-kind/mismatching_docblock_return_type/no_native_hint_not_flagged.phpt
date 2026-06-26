===description===
MismatchingDocblockReturnType does NOT fire when there is no native return
type hint — only the docblock @return is present.
===file===
<?php
/** @return int */
function onlyDocblockReturn() { return 42; }

/** @return string|null */
function onlyDocblockReturnNullable() { return null; }
===expect===
