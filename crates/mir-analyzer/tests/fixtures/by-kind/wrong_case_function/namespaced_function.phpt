===description===
Namespaced function with wrong-case short name is reported.
===config===
suppress=UnusedVariable
===file===
<?php
namespace App\Utils;
function formatDate(string $d): string { return $d; }

namespace App;
$x = \App\Utils\FORMATDATE("2024-01-01");
===expect===
WrongCaseFunction@6:5-6:26: Function name 'FORMATDATE' has incorrect casing; use 'formatDate'
