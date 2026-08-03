===description===
M18: gc_status()'s #[ArrayShape] only listed the PHP 7.3 keys (runs/
collected/threshold/roots, all int), so the PHP 8.x bool keys (running/
protected/full) fell back to int (the shared type of the listed keys)
instead of their real bool type, flagging a bogus InvalidArgument for a
bool-typed param.
===config===
suppress=UnusedParam
===file===
<?php
declare(strict_types=1);

function needsBool(bool $b): void {}

$status = gc_status();
needsBool($status['running']);
needsBool($status['protected']);
needsBool($status['full']);
===expect===
