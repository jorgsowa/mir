===config===
find_dead_code=true
===file===
<?php
function helper(): void {}

namespace App;

\helper();
===expect===
