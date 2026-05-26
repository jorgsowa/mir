===description===
Qualified name without a matching use import is resolved by prepending the current namespace.
Http\Request in namespace App means App\Http\Request, which PSR-4 maps to src/Http/Request.php.
===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:src/Http/Request.php===
<?php
namespace App\Http;
class Request {
    public function path(): string { return '/'; }
}
===file:Consumer.php===
<?php
namespace App;
function handle(): string {
    $r = new Http\Request();
    return $r->path();
}
===expect===
