===description===
Namespaced function with wrong-case short name is reported.
===file===
<?php
namespace App\Utils;
function formatDate(string $d): string { return $d; }

namespace App;
$x = \App\Utils\FORMATDATE("2024-01-01");
===expect===
WrongCaseFunction@6:6-6:27: Function name 'FORMATDATE' has incorrect casing; use 'formatDate'
