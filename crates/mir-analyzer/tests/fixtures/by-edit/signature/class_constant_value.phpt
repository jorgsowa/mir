===description===
Changing a class constant's value re-infers its type at the use site.
===file:Cfg.php===
<?php
class Cfg { const LIMIT = 10; }
===file:Use.php===
<?php
function run(): int { return Cfg::LIMIT; }
===expect===
<<none>>
===edit:Cfg.php===
<?php
class Cfg { const LIMIT = 'ten'; }
===expect===
Use.php: InvalidReturnType@2:22-2:40: Return type '"ten"' is not compatible with declared 'int'
