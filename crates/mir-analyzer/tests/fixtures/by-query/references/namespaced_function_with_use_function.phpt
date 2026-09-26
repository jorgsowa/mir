===description===
References to a namespaced function include calls through `use function` and fully-qualified calls.
===cursor===
references
===file:src/helpers.php===
<?php
namespace App\Support;

function helper(): int { return 1; }
===file:main.php===
<?php
use function App\Support\helper;

echo hel<CURSOR>per();
echo \App\Support\helper();
===expect===
main.php@4:5-4:11
main.php@5:5-5:24
