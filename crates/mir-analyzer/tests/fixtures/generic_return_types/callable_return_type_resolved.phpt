===description===
callable return type resolved
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
    $resolver()->undefinedMethod();
}
===expect===
UndefinedMethod@14:4: Method Service::undefinedMethod() does not exist
===ignore===
TODO
