===description===
Abstract class instantiation
===config===
suppress=MissingReturnType
===file===
<?php
abstract class AbstractService {
    abstract public function run();
}

/**
 * @param class-string<AbstractService> $serviceName
 */
function createService($serviceName) {
    return new $serviceName();
}

// `class-string<AbstractService>` means the caller passes a concrete subclass,
// so `new $serviceName()` is valid — AbstractInstantiation is not emitted here.
createService(AbstractService::class);
===expect===
