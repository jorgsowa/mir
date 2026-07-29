===description===
A PEAR-style (no-namespace, underscore-separated) class reachable only via a
`psr-0` autoload entry is lazily resolvable by FQCN. The underscore in the
class name maps to a directory separator per PSR-0's PEAR convention.
===file:composer.json===
{"autoload":{"psr-0":{"Old_":"src/"}}}
===file:src/Old/Thing.php===
<?php
class Old_Thing {
    public function name(): string { return 'thing'; }
}
===file:Handler.php===
<?php
class Handler {
    public function handle(Old_Thing $t): string {
        return $t->name();
    }
}
===expect===
