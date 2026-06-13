===description===
Class string parameter with type hint
===config===
suppress=MissingReturnType
===file===
<?php
class Service {
    public function run() {
        return "ok";
    }
}

/**
 * @param class-string<Service> $serviceName
 */
function loadService(string $serviceName) {
    $service = new $serviceName();
    return $service->run();
}

loadService(Service::class);
===expect===
