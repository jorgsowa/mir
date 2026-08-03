===description===
M18 follow-up: gc_status()'s bool/float keys (running/protected/full/
buffer_size/application_time/collector_time/destructor_time/free_time)
were only added in PHP 8.3.0 — targeting an older version via
#[LanguageLevelTypeAware] must keep the original 4-key int-only shape, so
reading 'running' on that target is a genuine NonExistentArrayOffset, not
a bool.
===config===
php_version=8.0
suppress=UnusedParam,MixedArgument
===file===
<?php
function needsBool(bool $x): void {}

$status = gc_status();
needsBool($status['running']);
===expect===
NonExistentArrayOffset@5:18: Array offset 'running' does not exist
