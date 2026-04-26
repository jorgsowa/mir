===file===
<?php
function takes_nullable(?string $s): void { var_dump($s); }

takes_nullable(null);
===expect===
