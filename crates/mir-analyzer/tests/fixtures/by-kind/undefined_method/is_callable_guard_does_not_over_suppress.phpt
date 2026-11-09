===description===
An is_callable() guard doesn't prove the callable in its false branch, after the if, or for another method or receiver.
===file===
<?php
class C {
    public function call(callable|string $cb): mixed { return $cb; }
}

function f(C $k, Throwable $e, Throwable $other): void {
    if (!is_callable([$e, 'report'])) {
        $k->call([$e, 'report']);
//               ^^^^^^^^^^^^^^ UndefinedMethod: Method Throwable::report() does not exist
    }
    if (is_callable([$e, 'report'])) {
        $k->call([$e, 'render']);
//               ^^^^^^^^^^^^^^ UndefinedMethod: Method Throwable::render() does not exist
        $k->call([$other, 'report']);
//               ^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Throwable::report() does not exist
    }
    $k->call([$e, 'report']);
//           ^^^^^^^^^^^^^^ UndefinedMethod: Method Throwable::report() does not exist
}
