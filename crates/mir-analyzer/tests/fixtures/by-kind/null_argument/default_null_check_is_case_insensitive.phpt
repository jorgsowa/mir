===description===
`= NULL` counts as a null default.
===config===
suppress=UnusedParam
===file===
<?php
/** @param string $f */
function plain($f = NULL): void {}
plain(null);
===expect===
