===description===
A local `@psalm-type` alias naming exception classes expands in `@throws` of
methods and functions instead of being treated as a class `Ns\Alias`.
===config===
suppress=UnusedParam
===file===
<?php
namespace App;

class MyEx extends \Exception {}
class OtherEx extends \Exception {}

/** @psalm-type Failure = MyEx|OtherEx */
class Svc {
    /** @throws Failure */
    public function run(): void { throw new MyEx(); }
}

/**
 * @psalm-type Single = MyEx
 * @throws Single
 */
function go(): void { throw new MyEx(); }
===expect===
