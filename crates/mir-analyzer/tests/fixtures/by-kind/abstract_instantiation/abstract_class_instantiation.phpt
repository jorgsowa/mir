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

// SHOULD emit AbstractInstantiation because AbstractService is abstract
createService(AbstractService::class);
===expect===
AbstractInstantiation@10:15-10:27: Cannot instantiate abstract class AbstractService
