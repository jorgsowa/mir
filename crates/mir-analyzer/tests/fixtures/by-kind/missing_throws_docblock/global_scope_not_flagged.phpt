===description===
top-level code has no docblock to declare @throws in
===file===
<?php
/**
 * @throws \Exception
 */
function riskyOperation(): void {
    throw new \Exception('fail');
}

class Service {
    /** @throws \Exception */
    public function run(): void {}
}

riskyOperation();
(new Service())->run();
throw new \Exception('fail');
