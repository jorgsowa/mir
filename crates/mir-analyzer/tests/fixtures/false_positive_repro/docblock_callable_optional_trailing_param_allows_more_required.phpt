===description===
A docblock callable type's `=` suffix (`Closure(mixed, array=, string=):mixed`) marks a
trailing param the implementing closure is free to leave out of its own signature — it is
not a promise that invocations always omit it. PHP callables may always declare more
required params than an optional-trailing-param marker implies; mir previously flagged this
as InvalidArgument by comparing against only the non-`=` param count instead of the full
declared signature.
===config===
suppress=UnusedParam
===file===
<?php
/** @param Closure(mixed, array=, string=):mixed $cb */
function register(Closure $cb): void {}

register(function (int $a, array $b, string $c): void {});

===expect===
