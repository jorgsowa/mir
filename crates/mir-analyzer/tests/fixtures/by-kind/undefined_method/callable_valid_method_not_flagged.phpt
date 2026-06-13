===description===
callable valid method not flagged
===config===
suppress=MissingClosureReturnType
===file===
<?php
/**
 * @template T
 */
class Registry {
    /** @return callable(): T */
    public function resolver(): callable { return function() { return null; }; }
}
class Service { public function handle(): void {} }
function test(): void {
    /** @var Registry<Service> $reg */
    $reg = new Registry();
    $resolver = $reg->resolver();
    $resolver()->handle();
}
===expect===
