===description===
static call via bare FQN on a PSR-4 lazy-loaded class produces no error
===file:composer.json===
{"autoload":{"psr-4":{"Util\\":"lib/"}}}
===file:lib/Formatter.php===
<?php
namespace Util;
class Formatter {
    public static function format(string $value): string { return strtoupper($value); }
}
===file:App.php===
<?php
function acceptString(string $s): void { var_dump($s); }
function run(): void {
    acceptString(\Util\Formatter::format('hello'));
}
===expect===
