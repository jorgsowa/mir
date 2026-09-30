===description===
mir-check on undefined variable
===file===
<?php
/** @mir-check $undefined is string */
echo "test";
//<^^^^^^^^^^^^ TypeCheckMismatch: Type of $undefined is expected to be string, got mixed
===expect===
