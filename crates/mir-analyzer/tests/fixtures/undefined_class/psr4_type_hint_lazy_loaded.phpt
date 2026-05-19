===description===
A class used only as a parameter type hint (no new, no static call, no use import) is
pre-loaded via PSR-4 so Pass-2 does not emit a false-positive UndefinedClass.
===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:src/Request.php===
<?php
namespace App;
class Request {
    public function path(): string { return '/'; }
}
===file:Handler.php===
<?php
namespace App;
class Handler {
    public function handle(Request $r): string {
        return $r->path();
    }
}
===expect===
