===description===
mir-check on undefined variable
===file===
<?php
/** @mir-check $undefined is string */
echo "test";
===expect===
TypeCheckMismatch@3:1: Type of $undefined is expected to be string, got mixed
