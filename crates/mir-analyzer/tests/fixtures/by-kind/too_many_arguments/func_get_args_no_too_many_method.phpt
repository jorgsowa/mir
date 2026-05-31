===description===
Method using func_get_args() accepts extra positional args without TooManyArguments
===file===
<?php
trait AcceptsMany {
    public function only($keys) {
        return is_array($keys) ? $keys : func_get_args();
    }
}

class Request {
    use AcceptsMany;
}

$r = new Request();
$r->only('email', 'password');
$r->only('a', 'b', 'c', 'd');
===expect===
