===description===
Passing a value that does not satisfy a method-level @psalm-type alias triggers InvalidArgument
===config===
suppress=UnusedParam
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
===expect===
InvalidArgument@13:10-13:17: Argument $method of route() expects '"GET"|"POST"|"PUT"|"DELETE"', got '"PATCH"'
