===description===
Qualified name (contains backslash, no leading backslash, no matching import) is resolved
by prepending the current namespace — Http\Request in namespace App means App\Http\Request.
===file:Http/Request.php===
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
