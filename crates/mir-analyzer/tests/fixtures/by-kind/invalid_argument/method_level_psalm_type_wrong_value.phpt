===description===
Passing a value that does not satisfy a method-level @psalm-type alias triggers InvalidArgument
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

class Router {
    /**
     * @psalm-type HttpMethod = "GET"|"POST"|"PUT"|"DELETE"
     * @param HttpMethod $method
     */
    public function route(string $method): void {}
}

$r = new Router();
$r->route("PATCH");
//        ^^^^^^^ InvalidArgument: Argument $method of route() expects '"GET"|"POST"|"PUT"|"DELETE"', got '"PATCH"'
