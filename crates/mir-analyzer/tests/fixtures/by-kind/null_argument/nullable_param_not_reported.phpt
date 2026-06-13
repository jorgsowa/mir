===description===
nullable param not reported
===config===
suppress=ForbiddenCode
===file===
<?php
function takes_nullable(?string $s): void { var_dump($s); }

takes_nullable(null);
===expect===
