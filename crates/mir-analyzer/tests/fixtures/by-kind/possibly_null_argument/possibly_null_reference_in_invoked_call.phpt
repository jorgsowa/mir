===description===
Possibly null reference in invoked call
===file===
<?php
interface Location {
    public function getId(): int;
}

/** @immutable */
interface Application {
    public function getLocation(): ?Location;
}

interface TakesId {
    public function __invoke(int $location): int;
}

function f(TakesId $takesId, Application $application): void {
   ($takesId)($application->getLocation()->getId());
}
===expect===
PossiblyNullMethodCall@16:15-16:51: Cannot call method getId() on possibly null value
