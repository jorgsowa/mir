===description===
Wrong case in namespace prefix segment of a function call is reported.
===config===
suppress=UnusedVariable
===file===
<?php
namespace MyApp\Utils;
function formatDate(string $d): string { return $d; }

namespace Client;
$x = \myapp\utils\formatDate("2024-01-01");
===expect===
WrongCaseFunction@6:5-6:28: Function name 'myapp\utils\formatDate' has incorrect casing; use 'MyApp\Utils\formatDate'
