===description===
\Closure used as a fully-qualified type hint in a namespaced file should not
be mis-resolved to Namespace\Closure. Passing a closure literal to a \Closure
parameter should not produce InvalidArgument.
===php_version===
8.0
===config===
suppress=MissingClosureReturnType,UnusedParam
===file===
<?php
namespace Foo\Bar;

function acceptsClosure(\Closure $callback): void {}

acceptsClosure(function() {});
===expect===
