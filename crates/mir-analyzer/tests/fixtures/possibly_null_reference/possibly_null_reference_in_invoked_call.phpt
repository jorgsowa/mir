===description===
possiblyNullReferenceInInvokedCall
===file===
<?php
interface Location {
    public function getId(): int;
}

/** @psalm-immutable */
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
PossiblyNullReference
===ignore===
TODO
