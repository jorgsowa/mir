===description===
Namespaced function with wrong-case short name is reported.
===file===
<?php
namespace App\Utils;
function formatDate(string $d): string { return $d; }

namespace App;
\App\Utils\FORMATDATE("2024-01-01");
===expect===
WrongCaseFunction@6:1-6:22: Function name 'FORMATDATE' has incorrect casing; use 'formatDate'
