===description===
method call on instance created via bare FQN without use statement produces no error; class is discovered via PSR-4 lazy loading, not pre-loaded
===config===
suppress=ForbiddenCode
===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:src/Builder.php===
<?php
namespace App;
class Builder {
    public function build(): string { return 'result'; }
}
===file:Factory.php===
<?php
function requireString(string $s): void { var_dump($s); }
function make(): void {
    requireString((new \App\Builder())->build());
}
===expect===
