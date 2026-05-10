===description===
php cs fixer config namespaces
===file:composer.json===
{
  "autoload-dev": {
    "psr-4": {
      "StubTests\\": "tests/"
    }
  }
}
===file:.php-cs-fixer.php===
<?php
declare(strict_types=1);

use StubTests\CodeStyle\BracesOneLineFixer;

$fixer = new BracesOneLineFixer();

return [
    'fixers' => [$fixer],
    'rules' => [
        'clean_namespace' => true,
        'no_leading_namespace_whitespace' => true,
        'no_unneeded_braces' => ['namespaces' => true],
        'blank_line_after_namespace' => true,
        'blank_lines_before_namespace' => true,
    ],
];
===file:tests/CodeStyle/BracesOneLineFixer.php===
<?php
namespace StubTests\CodeStyle;

class BracesOneLineFixer {}
===expect===
